use crate::model::Config;
use crate::{
    model::{activity, atomic_json, err, now, Installation, Result, Store},
    release::{self, Release},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use tauri::{Emitter, Manager};
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub engine: String,
    pub version: String,
    pub status: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed: u64,
    pub error: String,
    #[serde(skip)]
    pub cancel: Arc<AtomicBool>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub engine: String,
    pub version: String,
    pub executable: String,
    pub files: BTreeMap<String, String>,
}
pub fn hash(p: &Path) -> Result<String> {
    let mut f = fs::File::open(p).map_err(err)?;
    let mut h = Sha256::new();
    let mut buf = [0; 65536];
    loop {
        let n = f.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn verify_download(p: &Path, expected: Option<&str>) -> Result<()> {
    let expected = expected.ok_or("The official release provides no SHA-256 checksum. Use the official project to review and import it manually.")?;
    if expected.len() != 64 || !expected.bytes().all(|b|b.is_ascii_hexdigit()) || !hash(p)?.eq_ignore_ascii_case(expected) {
        return Err("SHA-256 verification failed. The archive was not installed.".into());
    }
    Ok(())
}
pub fn safe_relative(name: &str) -> Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    if normalized.is_empty() || normalized.starts_with('/') || normalized.contains(':') {
        return Err("Archive contains an unsafe path.".into());
    }
    let mut out = PathBuf::new();
    for p in normalized.split('/') {
        if p.is_empty() {
            continue;
        }
        let stem = p.split('.').next().unwrap_or("").to_uppercase();
        if p == "."
            || p == ".."
            || p.ends_with(['.', ' '])
            || p.chars().any(|c| c < ' ' || "<>\"|?*".contains(c))
            || [
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9",
            ]
            .contains(&stem.as_str())
        {
            return Err("Archive contains an unsafe Windows filename.".into());
        }
        out.push(p);
    }
    if out.as_os_str().is_empty() {
        return Err("Empty archive path".into());
    }
    Ok(out)
}
pub fn is_link(p: &Path) -> Result<bool> {
    let m = fs::symlink_metadata(p).map_err(err)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Ok(m.file_attributes() & 0x400 != 0)
    }
    #[cfg(not(windows))]
    {
        Ok(m.file_type().is_symlink())
    }
}
pub fn no_links(p: &Path) -> Result<()> {
    for ancestor in p.ancestors() {
        if ancestor.exists() && is_link(ancestor)? {
            return Err("Linked folders are not supported for managed installations. Choose a normal directory.".into());
        }
    }
    Ok(())
}
pub fn files(root: &Path) -> Result<Vec<PathBuf>> {
    fn walk(p: &Path, out: &mut Vec<PathBuf>, depth: usize) -> Result<()> {
        if depth > 64 || out.len() > 100000 {
            return Err("Installation exceeds safe file limits.".into());
        }
        for e in fs::read_dir(p).map_err(err)? {
            let p = e.map_err(err)?.path();
            if is_link(&p)? {
                return Err("Installation contains a link or junction; operation stopped.".into());
            }
            if p.is_dir() {
                walk(&p, out, depth + 1)?
            } else {
                out.push(p)
            }
        }
        Ok(())
    }
    let mut out = vec![];
    walk(root, &mut out, 0)?;
    Ok(out)
}
fn copy_entry(reader: &mut dyn Read, writer: &mut dyn Write, budget: &mut u64) -> Result<()> {
    let copied = std::io::copy(&mut reader.take(*budget + 1), writer).map_err(err)?;
    if copied > *budget { return Err("Archive exceeds actual extraction limit.".into()); }
    *budget -= copied;
    Ok(())
}
pub fn extract(archive: &Path, dest: &Path, cancel: &AtomicBool) -> Result<()> {
    let mut total = 0u64;
    let mut budget = 8 * 1024 * 1024 * 1024u64;
    let mut count = 0usize;
    let mut names = std::collections::HashSet::new();
    if archive.extension().and_then(|s| s.to_str()) == Some("zip") {
        let mut z = zip::ZipArchive::new(fs::File::open(archive).map_err(err)?).map_err(err)?;
        if z.len() > 100000 {
            return Err("Too many archive entries".into());
        }
        for i in 0..z.len() {
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            let mut e = z.by_index(i).map_err(err)?;
            let rel = safe_relative(&e.name().map_err(err)?)?;
            if !names.insert(rel.to_string_lossy().to_lowercase()) {
                return Err("Duplicate archive path".into());
            }
            if e.unix_mode()
                .map(|m| m & 0o170000 == 0o120000)
                .unwrap_or(false)
            {
                return Err("Archive links are not allowed.".into());
            }
            total = total.checked_add(e.size()).ok_or("Archive size overflow")?;
            if total > 8 * 1024 * 1024 * 1024 {
                return Err("Archive exceeds 8 GiB extraction limit.".into());
            }
            let p = dest.join(rel);
            if e.is_dir() {
                fs::create_dir_all(&p).map_err(err)?
            } else {
                fs::create_dir_all(p.parent().unwrap()).map_err(err)?;
                let mut f = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(p)
                    .map_err(err)?;
                copy_entry(&mut e, &mut f, &mut budget)?;
            }
        }
    } else {
        let source = fs::File::open(archive).map_err(err)?;
        // The library takes KiB and applies this cap to header buffers and decoder models.
        let mut seven = sevenz_rust::ArchiveReader::new_with_memory_limit(source, sevenz_rust::Password::empty(), 256 * 1024).map_err(err)?;
        seven.set_thread_count(1);
        seven.for_each_entries(|entry, reader| {
            let fail = |s: &str| sevenz_rust::Error::Other(s.to_string().into());
            if cancel.load(Ordering::Relaxed) {
                return Err(fail("Cancelled"));
            }
            count += 1;
            total = total.saturating_add(entry.size);
            if count > 100000 || total > 8 * 1024 * 1024 * 1024 {
                return Err(fail("Archive exceeds safety limits"));
            }
            if entry.windows_attributes & 0x400 != 0
                || (entry.windows_attributes >> 16) & 0o170000 == 0o120000
            {
                return Err(fail("Archive reparse points are forbidden"));
            }
            let rel = safe_relative(&entry.name).map_err(|e| fail(&e))?;
            if !names.insert(rel.to_string_lossy().to_lowercase()) {
                return Err(fail("Duplicate archive path"));
            }
            let p = dest.join(rel);
            if entry.is_directory {
                fs::create_dir_all(&p).map_err(sevenz_rust::Error::from)?
            } else {
                fs::create_dir_all(p.parent().unwrap()).map_err(sevenz_rust::Error::from)?;
                let mut f = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&p)
                    .map_err(sevenz_rust::Error::from)?;
                copy_entry(reader, &mut f, &mut budget).map_err(|e|fail(&e))?;
            }
            Ok(true)
        })
        .map_err(err)?;
    }
    Ok(())
}
pub fn owned(i: &Installation) -> Result<Manifest> {
    if !i.managed {
        return Err(
            "Imported installations are user-owned. Manage them in their original launcher.".into(),
        );
    }
    let root = Path::new(&i.root);
    no_links(root)?;
    let m: Manifest =
        serde_json::from_slice(&fs::read(root.join(".patchwork-install.json")).map_err(err)?)
            .map_err(err)?;
    if m.engine != i.engine
        || root.join(safe_relative(&m.executable)?) != PathBuf::from(&i.executable)
    {
        return Err("Ownership record does not match. No files were changed.".into());
    }
    Ok(m)
}
fn preserve(old: &Path, new: &Path, m: &Manifest) -> Result<()> {
    for f in files(old)? {
        let rel = f.strip_prefix(old).map_err(err)?;
        let key = rel.to_string_lossy().replace('\\', "/");
        if key == ".patchwork-install.json" {
            continue;
        }
        let ext = f
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let customized = m.files.get(&key).is_some_and(|h| {
            ["json", "ini", "yml", "yaml", "toml", "cfg"].contains(&ext.as_str())
                && hash(&f).ok().as_ref() != Some(h)
        });
        if !m.files.contains_key(&key) || customized {
            let to = new.join(rel);
            fs::create_dir_all(to.parent().unwrap()).map_err(err)?;
            fs::copy(&f, to).map_err(err)?;
        }
    }
    Ok(())
}
fn update(app: &tauri::AppHandle, engine: &str, f: impl FnOnce(&mut Task)) {
    let s = app.state::<Store>();
    let mut tasks = s.tasks.lock().unwrap();
    if let Some(t) = tasks.get_mut(engine) {
        f(t);
        let _ = app.emit("download", t.clone());
    }
}
#[derive(Serialize, Deserialize)]
struct Resume {
    url: String,
    etag: String,
}
fn download(
    app: &tauri::AppHandle,
    r: &Release,
    cache: &Path,
    cancel: &AtomicBool,
) -> Result<PathBuf> {
    fs::create_dir_all(cache).map_err(err)?;
    let key = format!(
        "{:x}",
        Sha256::digest(r.asset.browser_download_url.as_bytes())
    );
    let extension = if r.asset.name.ends_with(".zip") {
        "zip"
    } else {
        "7z"
    };
    let part = cache.join(format!("{key}.part"));
    let meta = cache.join(format!("{key}.json"));
    let finished = cache.join(format!("{key}.{extension}"));
    let c = release::client()?;
    let saved: Option<Resume> = fs::read(&meta)
        .ok()
        .and_then(|s| serde_json::from_slice(&s).ok());
    let offset = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    let resumable = saved
        .as_ref()
        .is_some_and(|s| s.url == r.asset.browser_download_url && s.etag.starts_with('"'))
        && offset > 0
        && offset < r.asset.size;
    let mut request = c
        .get(&r.asset.browser_download_url)
        .timeout(std::time::Duration::from_secs(7200));
    if resumable {
        request = request
            .header("Range", format!("bytes={offset}-"))
            .header("If-Range", &saved.as_ref().unwrap().etag)
    }
    let mut response = request
        .send()
        .map_err(err)?
        .error_for_status()
        .map_err(err)?;
    let resumed = resumable && response.status() == 206;
    if response.status() == 206 {
        let range = response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let expected = format!("bytes {offset}-");
        if !resumed
            || !range.starts_with(&expected)
            || !range.ends_with(&format!("/{}", r.asset.size))
        {
            return Err("Server returned an invalid resume range. Retry the download.".into());
        }
    }
    let etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    atomic_json(
        &meta,
        &Resume {
            url: r.asset.browser_download_url.clone(),
            etag,
        },
        false,
    )?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(!resumed)
        .append(resumed)
        .open(&part)
        .map_err(err)?;
    let initial = if resumed { offset } else { 0 };
    let mut bytes = initial;
    let start = Instant::now();
    let mut tick = Instant::now();
    let mut buffer = [0; 65536];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled; partial download retained for safe resume.".into());
        }
        let n = response.read(&mut buffer).map_err(err)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        if bytes > r.asset.size {
            return Err("Download exceeded advertised size.".into());
        }
        file.write_all(&buffer[..n]).map_err(err)?;
        if tick.elapsed().as_millis() > 150 {
            update(app, &r.engine, |t| {
                t.downloaded = bytes;
                t.speed =
                    ((bytes - initial) as f64 / start.elapsed().as_secs_f64().max(0.01)) as u64
            });
            tick = Instant::now();
        }
    }
    file.sync_all().map_err(err)?;
    drop(file);
    if bytes != r.asset.size {
        return Err("Incomplete download. Retry to resume safely.".into());
    }
    update(app, &r.engine, |t| {
        t.status = "Verifying".into();
        t.downloaded = bytes;
        t.speed = 0
    });
    let mut expected = r
        .asset
        .digest
        .as_deref()
        .and_then(|s| s.strip_prefix("sha256:"))
        .map(str::to_string);
    if expected.is_none() {
        if let Some(a) = &r.checksum {
            let prefix = format!(
                "https://github.com/{}/releases/download/",
                release::repo(&r.engine)?
            );
            if !a.browser_download_url.starts_with(&prefix) {
                return Err("Checksum source is not official.".into());
            }
            let text = c
                .get(&a.browser_download_url)
                .send()
                .map_err(err)?
                .error_for_status()
                .map_err(err)?
                .text()
                .map_err(err)?;
            expected = text.split_whitespace().next().map(str::to_string);
        }
    }
    verify_download(&part, expected.as_deref())?;
    if finished.exists() {
        fs::remove_file(&finished).map_err(err)?;
    }
    fs::rename(&part, &finished).map_err(err)?;
    Ok(finished)
}
pub fn start(app: tauri::AppHandle, engine: String) -> Result<()> {
    let s = app.state::<Store>();
    release::repo(&engine)?;
    if !s.running.lock().unwrap().is_empty() {
        return Err(
            "Close running games and engine settings before changing an installation.".into(),
        );
    }
    let mut busy = s.busy.lock().unwrap();
    if *busy {
        return Err("Another installation is in progress.".into());
    }
    *busy = true;
    let flag = Arc::new(AtomicBool::new(false));
    s.tasks.lock().unwrap().insert(
        engine.clone(),
        Task {
            engine: engine.clone(),
            version: String::new(),
            status: "Finding release".into(),
            downloaded: 0,
            total: 0,
            speed: 0,
            error: String::new(),
            cancel: flag.clone(),
        },
    );
    drop(busy);
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            install_release(&app, &engine, &flag)?;
            if engine=="rpcs3" {
                let c=app.state::<Store>().get();let i=c.installations.get("rpcs3").ok_or("RPCS3 installation reference missing")?;
                crate::ps3::ensure_firmware(&app.state::<Store>().dir,i,|message|update(&app,&engine,|t|t.status=message.into()),Some(&flag))?;
            }
            Ok(())
        }))
        .unwrap_or_else(|_| {
            Err(
                "Installation stopped after an internal error. Existing installation retained."
                    .into(),
            )
        });
        update(&app, &engine, |t| match result {
            Ok(()) => t.status = "Complete".into(),
            Err(e) => {
                t.status = if flag.load(Ordering::Relaxed) {
                    "Cancelled"
                } else {
                    "Failed"
                }
                .into();
                t.error = e
            }
        });
        *app.state::<Store>().busy.lock().unwrap() = false;
        let _ = app.emit("changed", ());
    });
    Ok(())
}
fn install_release(app: &tauri::AppHandle, engine: &str, cancel: &AtomicBool) -> Result<()> {
    let s = app.state::<Store>();
    let c = s.get();
    let existing = c.installations.get(engine);
    if let Some(i) = existing {
        owned(i)?;
    }
    let r = release::latest(engine)?;
    update(app, engine, |t| {
        t.version = r.version.clone();
        t.total = r.asset.size;
        t.status = "Downloading".into()
    });
    let archive = download(app, &r, &s.dir.join("downloads"), cancel)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled".into());
    }
    let parent = PathBuf::from(&c.install_root);
    no_links(&parent)?;
    fs::create_dir_all(&parent).map_err(err)?;
    let dest = existing
        .map(|i| PathBuf::from(&i.root))
        .unwrap_or(parent.join(engine));
    for g in &c.games {
        for data in [&g.data, &g.update] {
            if !data.is_empty() {
                let p = Path::new(data).canonicalize().map_err(err)?;
                let base = dest
                    .parent()
                    .unwrap()
                    .canonicalize()
                    .map_err(err)?
                    .join(dest.file_name().unwrap());
                if p.starts_with(&base) || base.starts_with(&p) {
                    return Err(
                        "Installation folders must be separate from original game data.".into(),
                    );
                }
            }
        }
    }
    if dest.exists() && existing.is_none() {
        return Err("Destination already exists and is not owned by this launcher.".into());
    }
    let stage = tempfile::Builder::new()
        .prefix(".patchwork-stage-")
        .tempdir_in(dest.parent().unwrap())
        .map_err(err)?;
    update(app, engine, |t| t.status = "Extracting".into());
    extract(&archive, stage.path(), cancel)?;
    let expected = match engine {
        "lbppc" => Some("lbppc.exe"),
        "rpcs3" => Some("rpcs3.exe"),
        _ => None,
    };
    let exes: Vec<_> = files(stage.path())?
        .into_iter()
        .filter(|p| {
            let n = p.file_name().unwrap().to_string_lossy().to_lowercase();
            expected
                .map(|e| n == e)
                .unwrap_or(n.ends_with(".exe") && !n.contains("unins"))
        })
        .collect();
    if exes.len() != 1 {
        return Err(
            "Could not identify a unique official Windows executable. Installation left unchanged."
                .into(),
        );
    }
    crate::metadata::windows_exe(&exes[0])?;
    let payload = exes[0].parent().unwrap();
    let rel_exe = exes[0].file_name().unwrap().to_string_lossy().to_string();
    let mut manifest = Manifest {
        engine: engine.into(),
        version: r.version.clone(),
        executable: rel_exe.clone(),
        files: BTreeMap::new(),
    };
    for f in files(payload)? {
        manifest.files.insert(
            f.strip_prefix(payload)
                .map_err(err)?
                .to_string_lossy()
                .replace('\\', "/"),
            hash(&f)?,
        );
    }
    if let Some(i) = existing {
        preserve(Path::new(&i.root), payload, &owned(i)?)?;
    }
    atomic_json(&payload.join(".patchwork-install.json"), &manifest, false)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled before installation.".into());
    }
    update(app, engine, |t| t.status = "Installing".into());
    let backup = dest.with_file_name(format!("{engine}-backup-{}", now()));
    if dest.exists() {
        fs::rename(&dest, &backup).map_err(err)?;
    }
    if let Err(e) = fs::rename(payload, &dest) {
        if backup.exists() {
            let _ = fs::rename(&backup, &dest);
        }
        return Err(err(e));
    }
    let installed = Installation {
        engine: engine.into(),
        executable: dest.join(&rel_exe).to_string_lossy().into(),
        version: r.version,
        managed: true,
        root: dest.to_string_lossy().into(),
        rollback: if backup.exists() {
            backup.to_string_lossy().into()
        } else {
            String::new()
        },
    };
    if let Err(e) = s.change(|c| {
        c.installations.insert(engine.into(), installed);
        activity(c, &format!("{engine} installed from its official release."));
        Ok(())
    }) {
        let failed = dest.with_file_name(format!("{engine}-uncommitted-{}", now()));
        let _ = fs::rename(&dest, failed);
        if backup.exists() {
            let _ = fs::rename(&backup, &dest);
        }
        return Err(e);
    }
    Ok(())
}
fn relocate_game_paths(c: &mut Config, engine: &str, old: &Path, canonical_old: &Path, new: &Path) {
    for game in c.games.iter_mut().filter(|g|g.engine==engine) {
        for value in [&mut game.data,&mut game.update] {
            if value.is_empty(){continue;}
            let path=Path::new(value);
            if let Ok(tail)=path.strip_prefix(canonical_old).or_else(|_|path.strip_prefix(old)) {
                *value=new.join(tail).to_string_lossy().into();
            }
        }
    }
}
pub fn maintenance(s: &Store, engine: &str, action: &str, target: Option<&str>) -> Result<String> {
    let c = s.get();
    let i = c
        .installations
        .get(engine)
        .ok_or("No installation configured")?;
    if action == "forget" {
        if i.managed {
            return Err("Use Uninstall for managed components.".into());
        }
        s.change(|c| {
            c.installations.remove(engine);
            Ok(())
        })?;
        return Ok("Installation reference removed. Files were untouched.".into());
    }
    let m = owned(i)?;
    let root = Path::new(&i.root);
    let canonical_root=root.canonicalize().map_err(err)?;
    if action == "verify" {
        let mut damaged = vec![];
        for (name, h) in &m.files {
            let p = root.join(safe_relative(name)?);
            if hash(&p).ok().as_ref() != Some(h) {
                damaged.push(name.clone())
            }
        }
        return Ok(if damaged.is_empty() {
            "All installed release files match their recorded SHA-256 hashes.".into()
        } else {
            format!("{} changed or missing files. Use Repair to reinstall official files; customized settings are preserved.\n{}",damaged.len(),damaged.join("\n"))
        });
    }
    if action == "uninstall" {
        let retained = root.with_file_name(format!("{engine}-uninstalled-{}", now()));
        fs::rename(root, &retained).map_err(err)?;
        if let Err(e) = s.change(|c| {
            relocate_game_paths(c,engine,root,&canonical_root,&retained);
            c.installations.remove(engine);
            activity(
                c,
                &format!("{engine} uninstalled; saves and files retained in backup."),
            );
            Ok(())
        }) {
            let _ = fs::rename(&retained, root);
            return Err(e);
        }
        return Ok(format!(
            "Uninstalled. All files and saves retained at {}",
            retained.display()
        ));
    }
    if action == "move" {
        let parent = PathBuf::from(target.ok_or("Choose a destination")?);
        no_links(&parent)?;
        let parent = parent.canonicalize().map_err(err)?;
        let dest = parent.join(engine);
        if dest.exists() || dest.starts_with(root) || root.starts_with(&dest) {
            return Err("Choose an empty, separate destination.".into());
        }
        for g in &c.games {
            for data in [&g.data, &g.update] {
                if !data.is_empty() {
                    let p = Path::new(data).canonicalize().map_err(err)?;
                    if dest.starts_with(&p) || p.starts_with(&dest) {
                        return Err("Destination overlaps game data.".into());
                    }
                }
            }
        }
        let stage = tempfile::Builder::new()
            .prefix(".patchwork-move-")
            .tempdir_in(&parent)
            .map_err(err)?;
        for f in files(root)? {
            let to = stage.path().join(f.strip_prefix(root).map_err(err)?);
            fs::create_dir_all(to.parent().unwrap()).map_err(err)?;
            fs::copy(&f, &to).map_err(err)?;
            if hash(&f)? != hash(&to)? {
                return Err("Move verification failed; original retained.".into());
            }
        }
        fs::rename(stage.path(), &dest).map_err(err)?;
        s.change(|c| {
            relocate_game_paths(c,engine,root,&canonical_root,&dest);
            let i = c.installations.get_mut(engine).unwrap();
            i.root = dest.to_string_lossy().into();
            i.executable = dest.join(&m.executable).to_string_lossy().into();
            Ok(())
        })?;
        return Ok(
            "Installation copied and verified at the new location. Original retained as a backup."
                .into(),
        );
    }
    if action == "rollback" {
        let previous = Path::new(&i.rollback);
        if !previous.is_dir() {
            return Err("No rollback is available.".into());
        }
        no_links(previous)?;
        let old: Manifest = serde_json::from_slice(
            &fs::read(previous.join(".patchwork-install.json")).map_err(err)?,
        )
        .map_err(err)?;
        if old.engine != engine {
            return Err("Rollback engine mismatch".into());
        }
        crate::metadata::windows_exe(&previous.join(safe_relative(&old.executable)?))?;
        preserve(root, previous, &m)?;
        let swap = root.with_file_name(format!("{engine}-before-rollback-{}", now()));
        fs::rename(root, &swap).map_err(err)?;
        if let Err(e) = fs::rename(previous, root) {
            let _ = fs::rename(&swap, root);
            return Err(err(e));
        }
        if let Err(e) = s.change(|c| {
            let i = c.installations.get_mut(engine).unwrap();
            i.version = old.version;
            i.rollback = swap.to_string_lossy().into();
            i.executable = root.join(&old.executable).to_string_lossy().into();
            Ok(())
        }) {
            let _ = fs::rename(root, previous);
            let _ = fs::rename(&swap, root);
            return Err(e);
        }
        return Ok("Previous release restored. Current user settings retained.".into());
    }
    Err("Unknown maintenance action".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn download_verification_requires_a_real_matching_checksum() {
        let file=tempfile::NamedTempFile::new().unwrap();
        fs::write(file.path(), b"official fixture").unwrap();
        assert!(verify_download(file.path(),None).is_err());
        assert!(verify_download(file.path(),Some(&"0".repeat(64))).is_err());
        assert!(verify_download(file.path(),Some(&hash(file.path()).unwrap())).is_ok());
    }
    #[test]
    fn extraction_bounds_actual_bytes_not_only_declared_sizes() {
        let mut reader=std::io::Cursor::new(b"four");let mut bytes=vec![];let mut budget=3;
        assert!(copy_entry(&mut reader,&mut bytes,&mut budget).is_err());
        assert_eq!(bytes.len(),4);
    }
    #[test]
    fn maintained_sevenz_reader_extracts_valid_archives() {
        let temp=tempfile::tempdir().unwrap();
        let source=temp.path().join("fixture.txt");
        fs::write(&source,b"local test fixture").unwrap();
        let archive=temp.path().join("fixture.7z");
        sevenz_rust::compress_to_path(&source,&archive).unwrap();
        let dest=tempfile::tempdir().unwrap();
        extract(&archive,dest.path(),&AtomicBool::new(false)).unwrap();
        assert_eq!(fs::read(dest.path().join("fixture.txt")).unwrap(),b"local test fixture");
    }
    #[test]
    fn sevenz_traversal_never_writes_outside_staging() {
        let temp=tempfile::tempdir().unwrap();
        let source=temp.path().join("fixture.txt");fs::write(&source,b"fixture").unwrap();
        let archive=temp.path().join("traversal.7z");
        let mut writer=sevenz_rust::ArchiveWriter::create(&archive).unwrap();
        writer.push_archive_entry(sevenz_rust::ArchiveEntry::from_path(&source,"../outside.txt".into()),Some(fs::File::open(&source).unwrap())).unwrap();
        writer.finish().unwrap();
        let dest=tempfile::tempdir_in(temp.path()).unwrap();
        assert!(extract(&archive,dest.path(),&AtomicBool::new(false)).is_err());
        assert!(!temp.path().join("outside.txt").exists());
    }
    #[test]
    fn extraction_rejects_zip_slip_and_extracts_valid_files() {
        let t = tempfile::tempdir().unwrap();
        for (name, valid) in [("bin/readme.txt", true), ("../outside.txt", false)] {
            let archive = t.path().join("test.zip");
            let mut z = zip::ZipWriter::new(fs::File::create(&archive).unwrap());
            z.start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            z.write_all(b"fixture").unwrap();
            z.finish().unwrap();
            let dest = tempfile::tempdir_in(t.path()).unwrap();
            assert_eq!(
                extract(&archive, dest.path(), &AtomicBool::new(false)).is_ok(),
                valid
            );
            if valid {
                assert_eq!(fs::read(dest.path().join(name)).unwrap(), b"fixture");
            }
            assert!(!t.path().join("outside.txt").exists());
        }
    }
    #[test]
    fn uninstall_retains_every_file_and_original_game_data() {
        let t = tempfile::tempdir().unwrap();
        let s = Store::open(t.path().join("config")).unwrap();
        let root = t.path().join("lbppc");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("save.dat"), b"precious").unwrap();
        let original = t.path().join("game-data");
        fs::create_dir(&original).unwrap();
        fs::write(original.join("owned.txt"), b"original").unwrap();
        atomic_json(
            &root.join(".patchwork-install.json"),
            &Manifest {
                engine: "lbppc".into(),
                version: "v1".into(),
                executable: "LBPPC.exe".into(),
                files: BTreeMap::new(),
            },
            false,
        )
        .unwrap();
        s.change(|c| {
            c.installations.insert(
                "lbppc".into(),
                Installation {
                    engine: "lbppc".into(),
                    root: root.to_string_lossy().into(),
                    executable: root.join("LBPPC.exe").to_string_lossy().into(),
                    managed: true,
                    ..Default::default()
                },
            );
            Ok(())
        })
        .unwrap();
        maintenance(&s, "lbppc", "uninstall", None).unwrap();
        assert!(!root.exists());
        assert!(!s.get().installations.contains_key("lbppc"));
        let backup = fs::read_dir(t.path())
            .unwrap()
            .flatten()
            .find(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("lbppc-uninstalled-")
            })
            .unwrap()
            .path();
        assert_eq!(fs::read(backup.join("save.dat")).unwrap(), b"precious");
        assert_eq!(fs::read(original.join("owned.txt")).unwrap(), b"original");
    }
    #[test]
    fn rejects_traversal_and_windows_aliases() {
        for s in [
            "../x",
            "/x",
            "C:\\x",
            "a/../../x",
            "a:stream",
            "CON.txt",
            "x./a",
            "a\\..\\b",
            "LPT1",
            "a ",
        ] {
            assert!(safe_relative(s).is_err(), "{s}");
        }
        assert_eq!(
            safe_relative("bin/runtime.dll").unwrap(),
            PathBuf::from("bin/runtime.dll")
        );
    }
    #[test]
    fn imported_uninstall_is_rejected() {
        assert!(owned(&Installation::default()).is_err())
    }
    #[test]
    fn checksum_is_real() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("a");
        fs::write(&p, b"abc").unwrap();
        assert_eq!(
            hash(&p).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
    #[test]
    fn preserves_user_files() {
        let t = tempfile::tempdir().unwrap();
        let old = t.path().join("old");
        let new = t.path().join("new");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&new).unwrap();
        fs::write(old.join("save.dat"), b"save").unwrap();
        preserve(
            &old,
            &new,
            &Manifest {
                engine: "lbppc".into(),
                version: "1".into(),
                executable: "LBPPC.exe".into(),
                files: BTreeMap::new(),
            },
        )
        .unwrap();
        assert_eq!(fs::read(new.join("save.dat")).unwrap(), b"save");
    }
    #[test] fn moving_engine_relinks_installed_games_but_not_external_dumps() {
        let mut c=Config::default();let old=Path::new("old-engine");let new=Path::new("new-engine");
        c.games[0].data=old.join("dev_hdd0/game/NPUA98148").to_string_lossy().into();
        c.games[1].data="external-disc".into();
        relocate_game_paths(&mut c,"rpcs3",old,old,new);
        assert_eq!(Path::new(&c.games[0].data),new.join("dev_hdd0/game/NPUA98148"));assert_eq!(c.games[1].data,"external-disc");
    }

}
