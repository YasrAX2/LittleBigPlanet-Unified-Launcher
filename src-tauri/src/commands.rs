use crate::{download, metadata, model::*, release};
use base64::Engine;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::Ordering,
};
use tauri::{Emitter, Manager};
fn requirements_cached(s: &Store) -> release::Requirements {
    fs::read(s.dir.join("requirements.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
fn idle(s: &Store) -> Result<()> {
    if *s.busy.lock().unwrap() || !s.running.lock().unwrap().is_empty() {
        Err("Close the running engine and finish downloads before changing files.".into())
    } else {
        Ok(())
    }
}
fn local_artwork(dir: &Path, name: &str, id: &str, limit: u64) -> Option<String> {
    if name.is_empty() || !valid_artwork(name, id) { return None; }
    let path = dir.join("art").join(name);
    download::no_links(&path).ok()?;
    let metadata = fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > limit { return None; }
    let bytes = fs::read(path).ok()?;
    let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") { "image/png" }
        else if bytes.starts_with(&[0xff, 0xd8, 0xff]) { "image/jpeg" } else { return None; };
    Some(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}
fn default_artwork(id: &str) -> Option<String> {
    let bytes: &[u8]=match id {
        "background" => include_bytes!("../assets/default-art/background.png"),
        "lbp1" => include_bytes!("../assets/default-art/lbp1.png"),
        "lbp2" => include_bytes!("../assets/default-art/lbp2.png"),
        "lbp3" => include_bytes!("../assets/default-art/lbp3.png"),
        "palette-amber-mint" => include_bytes!("../assets/default-art/palette-amber-mint.png"),
        "palette-black-green" => include_bytes!("../assets/default-art/palette-black-green.png"),
        "palette-blue-grey" => include_bytes!("../assets/default-art/palette-blue-grey.png"),
        "palette-cyan-yellow" => include_bytes!("../assets/default-art/palette-cyan-yellow.png"),
        "palette-olive-maroon" => include_bytes!("../assets/default-art/palette-olive-maroon.png"),
        "palette-peach-lavender" => include_bytes!("../assets/default-art/palette-peach-lavender.png"),
        "palette-pink-cream" => include_bytes!("../assets/default-art/palette-pink-cream.png"),
        "palette-purple-black" => include_bytes!("../assets/default-art/palette-purple-black.png"),
        "palette-red-burgundy" => include_bytes!("../assets/default-art/palette-red-burgundy.png"),
        "palette-violet-green" => include_bytes!("../assets/default-art/palette-violet-green.png"),
        _=>return None,
    };
    Some(format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes)))
}
#[tauri::command]
pub fn snapshot(s: tauri::State<Store>) -> Value {
    let c = s.get();
    let artwork: serde_json::Map<String,Value> = c.games.iter().filter_map(|g|local_artwork(&s.dir,&g.artwork,&g.id,8*1024*1024).or_else(||default_artwork(&g.id)).map(|v|(g.id.clone(),json!(v)))).collect();
    let palette_artwork: serde_json::Map<String,Value> = PALETTE_IDS.iter().filter_map(|id|{
        let key=format!("palette-{id}");
        local_artwork(&s.dir,&format!("{key}.png"),&key,64*1024).or_else(||default_artwork(&key)).map(|v|(id.to_string(),json!(v)))
    }).collect();
    let background_artwork=local_artwork(&s.dir,&c.background_artwork,"background",8*1024*1024).or_else(||default_artwork("background"));
    json!({"config":c,"backgroundArtwork":background_artwork,"paletteArtwork":palette_artwork,"tasks":s.tasks.lock().unwrap().values().cloned().collect::<Vec<_>>(),"running":*s.running.lock().unwrap(),"requirements":requirements_cached(&s),"artwork":artwork})
}
fn reset_look(c: &mut Config) {
    let defaults=Config::default();
    c.theme=defaults.theme;
    c.palette=defaults.palette;
    c.background=defaults.background;
    c.background_blur=defaults.background_blur;
    c.panel_opacity=defaults.panel_opacity;
    c.background_artwork.clear();
    for g in &mut c.games { g.artwork.clear(); }
}
#[tauri::command]
pub fn preferences(s: tauri::State<Store>, changes: Value) -> Result<Config> {
    s.change(|c| {
        if changes.get("resetLook").and_then(Value::as_bool)==Some(true) { reset_look(c); }
        if let Some(v) = changes.get("typography").and_then(Value::as_str) { c.typography=v.into(); }
        if let Some(v) = changes.get("theme").and_then(Value::as_str) {
            c.theme = v.into()
        }
        if let Some(v) = changes.get("palette").and_then(Value::as_str) {
            c.palette = v.into()
        }
        if let Some(v) = changes.get("background").and_then(Value::as_str) {
            c.background = v.into()
        }
        if let Some(v) = changes.get("backgroundBlur").and_then(Value::as_u64) {
            c.background_blur = u32::try_from(v).map_err(|_|"Invalid background blur")?
        }
        if let Some(v) = changes.get("panelOpacity").and_then(Value::as_u64) {
            c.panel_opacity = u32::try_from(v).map_err(|_|"Invalid panel opacity")?
        }
        if let Some(v) = changes.get("reducedMotion").and_then(Value::as_bool) {
            c.reduced_motion = v
        }
        if let Some(v) = changes.get("controller").and_then(Value::as_bool) {
            c.controller = v
        }
        if let Some(v) = changes.get("onboarding").and_then(Value::as_bool) {
            c.onboarding = v
        }
        if let Some(v) = changes.get("textScale").and_then(Value::as_u64) {
            c.text_scale = v as u32
        }
        if let Some(v) = changes.get("installRoot").and_then(Value::as_str) {
            idle(&s)?;
            let p = Path::new(v);
            if !p.is_absolute() || !p.is_dir() {
                return Err("Choose an existing absolute folder.".into());
            }
            download::no_links(p)?;
            c.install_root = p.canonicalize().map_err(err)?.to_string_lossy().into()
        }
        Ok(())
    })
}
#[tauri::command]
pub async fn pick(kind: String) -> Result<Option<String>> {
    tauri::async_runtime::spawn_blocking(move || {
        let d = rfd::FileDialog::new();
        let p = match kind.as_str() {
            "exe" => d
                .set_title("Choose the official Windows executable")
                .add_filter("Windows executable", &["exe"])
                .pick_file(),
            "pkg" => d.set_title("Choose a PS3 game or update package").add_filter("PS3 package", &["pkg"]).pick_file(),
            "pup" => d.set_title("Choose official PS3 firmware").add_filter("PS3 firmware", &["pup"]).pick_file(),
            "art" => d
                .set_title("Choose your own artwork")
                .add_filter("Artwork", &["png", "jpg", "jpeg"])
                .pick_file(),
            _ => d.set_title("Choose a folder").pick_folder(),
        };
        Ok(p.map(|p| p.to_string_lossy().into()))
    })
    .await
    .map_err(err)?
}
#[tauri::command]
pub fn import_data(
    s: tauri::State<Store>,
    id: String,
    path: String,
    update: bool,
) -> Result<metadata::Inspection> {
    idle(&s)?;
    let c = s.get();
    if !c.games.iter().any(|g| g.id == id) {
        return Err("Unknown game".into());
    }
    if update && id != "lbp3" {
        return Err("Install PS3 updates using RPCS3.".into());
    }
    let p = Path::new(&path);
    let r = metadata::inspect(&id, p, update, &requirements_cached(&s))?;
    for i in c.installations.values() {
        let root = Path::new(&i.root).canonicalize().map_err(err)?;
        let data = Path::new(&r.path);
        if data.starts_with(&root) || root.starts_with(data) {
            let title=r.title_id.as_bytes();
            let safe_title=title.len()==9 && title[..4].iter().all(u8::is_ascii_uppercase) && title[4..].iter().all(u8::is_ascii_digit);
            let installed=if i.engine=="rpcs3" && id!="lbp3" && safe_title { crate::ps3::root(i)?.join("dev_hdd0/game").join(&r.title_id).canonicalize().ok() }else{None};
            if installed.as_deref()!=Some(data) { return Err("Keep standalone game dumps separate from engines, or choose the exact installed RPCS3 game folder.".into()); }
        }
    }
    if !r.valid {
        return Err(r.messages.join("\n"));
    }
    s.change(|c| {
        let g = game_mut(c, &id)?;
        if update {
            if !g.title_id.is_empty() && g.title_id != r.title_id {
                return Err("Base game and update regions must match.".into());
            }
            g.update = r.path.clone()
        } else {
            g.data = r.path.clone();
            g.update = String::new();
            g.title_id = r.title_id.clone();
            g.region = r.region.clone();
            g.version = r.version.clone()
        }
        activity(
            c,
            &format!(
                "{} game files linked. Original files untouched.",
                id.to_uppercase()
            ),
        );
        Ok(())
    })?;
    Ok(r)
}
#[tauri::command]
pub fn clear_data(s: tauri::State<Store>, id: String, update: bool) -> Result<Config> {
    idle(&s)?;
    s.change(|c| {
        let g = game_mut(c, &id)?;
        g.update.clear();
        if !update {
            g.data.clear();
            g.title_id.clear();
            g.region.clear();
            g.version.clear()
        }
        Ok(())
    })
}
#[tauri::command]
pub fn import_engine(s: tauri::State<Store>, engine: String, path: String) -> Result<Config> {
    idle(&s)?;
    if !["lbppc", "rpcs3", "lbprecomp", "replanet"].contains(&engine.as_str()) {
        return Err("Unknown engine".into());
    }
    let p = Path::new(&path).canonicalize().map_err(err)?;
    metadata::windows_exe(&p)?;
    let name = p.file_name().unwrap().to_string_lossy().to_lowercase();
    if (engine == "lbppc" && name != "lbppc.exe") || (engine == "rpcs3" && name != "rpcs3.exe") {
        return Err("Choose LBPPC.exe or rpcs3.exe for the selected engine.".into());
    }
    let root = p.parent().unwrap();
    for g in &s.get().games {
        for data in [&g.data, &g.update] {
            if !data.is_empty() {
                let data = Path::new(data).canonicalize().map_err(err)?;
                if root.starts_with(&data) || data.starts_with(root) {
                    return Err(
                        "Choose an engine installation separate from original game data.".into(),
                    );
                }
            }
        }
    }
    let mut imported = Installation {
        engine: engine.clone(),
        executable: p.to_string_lossy().into(),
        root: root.to_string_lossy().into(),
        version: "Imported".into(),
        managed: false,
        rollback: String::new(),
    };
    if let Ok(bytes) = fs::read(root.join(".patchwork-install.json")) {
        if let Ok(m) = serde_json::from_slice::<download::Manifest>(&bytes) {
            if m.engine == engine {
                imported.version = m.version;
            }
        }
    }
    s.change(|c| {
        if c.installations.get(&engine).is_some_and(|i| i.managed) {
            return Err(
                "Uninstall or move the managed component before importing another copy.".into(),
            );
        }
        c.installations.insert(engine, imported);
        Ok(())
    })
}
#[tauri::command]
pub fn game_settings(s: tauri::State<Store>, id: String, changes: Value) -> Result<Config> {
    s.change(|c| {
        let g = game_mut(c, &id)?;
        if let Some(v) = changes.get("favorite").and_then(Value::as_bool) {
            g.favorite = v
        }
        if let Some(v)=changes.get("ps3Compatibility").and_then(Value::as_bool){g.ps3_compatibility=v;}
        if let Some(v) = changes.get("fullscreen").and_then(Value::as_bool) {
            g.fullscreen = v
        }
        if let Some(patch) = changes.get("video") {
            let patch=patch.as_object().ok_or("Video changes must be an object.")?;
            let mut value=serde_json::to_value(&g.video).map_err(err)?;
            for (key,value_patch) in patch { value.as_object_mut().unwrap().insert(key.clone(),value_patch.clone()); }
            g.video=serde_json::from_value(value).map_err(|_|"Invalid video settings.")?;
        }
        if let Some(v) = changes.get("engine").and_then(Value::as_str) {
            let allowed = match id.as_str() {
                "lbp1" => ["rpcs3", "lbprecomp"],
                "lbp2" => ["rpcs3", "replanet"],
                _ => ["lbppc", "lbppc"],
            };
            if !allowed.contains(&v) {
                return Err("That engine does not support this game.".into());
            }
            g.engine = v.into()
        }
        if !["rpcs3","lbppc"].contains(&g.engine.as_str()) { g.video.enabled=false; }
        Ok(())
    })
}
pub fn validate(s: &Store, id: &str) -> Result<metadata::Inspection> {
    let c = s.get();
    let g = c.games.iter().find(|g| g.id == id).ok_or("Unknown game")?;
    let req = requirements_cached(s);
    let mut r = metadata::inspect(id, Path::new(&g.data), false, &req)?;
    if id == "lbp3" {
        if !g.update.is_empty() {
            let u = metadata::inspect(id, Path::new(&g.update), true, &req)?;
            r.valid &= u.valid && u.title_id == r.title_id;
            r.messages.extend(u.messages);
            if u.title_id != r.title_id {
                r.messages
                    .push("Update region differs from the base game.".into())
            }
        } else if r.version.trim_start_matches('0') != req.version {
            r.valid = false;
            r.messages
                .push("The required update folder is missing.".into())
        }
    }
    let i = c.installations.get(&g.engine);
    if let Some(i) = i {
        if let Err(e) = metadata::windows_exe(Path::new(&i.executable)) {
            r.valid = false;
            r.messages.push(e)
        }
        if g.engine == "rpcs3" && !crate::ps3::firmware_present(i)? {
            r.valid=false;r.messages.push("PS3 firmware is missing. Choose Prepare RPCS3 to download, verify and install it automatically.".into());
        }
    } else {
        r.valid = false;
        r.messages
            .push("Install or import the selected engine first.".into())
    }
    Ok(r)
}
#[tauri::command]
pub fn validate_game(s: tauri::State<Store>, id: String) -> Result<metadata::Inspection> {
    validate(&s, &id)
}
#[tauri::command]
pub fn discover_games(s: tauri::State<Store>) -> Result<Vec<metadata::Inspection>> {
    let c = s.get();
    let i = c
        .installations
        .get("rpcs3")
        .ok_or("Import or install RPCS3 first.")?;
    let root = crate::ps3::root(i)?;
    let mut paths = vec![];
    if let Ok(entries) = fs::read_dir(root.join("dev_hdd0/game")) {
        for e in entries.flatten().take(5000) {
            if e.path().is_dir() {
                paths.push(e.path())
            }
        }
    }
    let yaml = root.join("games.yml");
    if yaml.is_file() && fs::metadata(&yaml).map_err(err)?.len() < 4 * 1024 * 1024 {
        let v: serde_yaml::Value =
            serde_yaml::from_slice(&fs::read(yaml).map_err(err)?).map_err(err)?;
        if let Some(m) = v.as_mapping() {
            for (_, v) in m {
                if let Some(p) = v.as_str() {
                    paths.push(PathBuf::from(p));
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    let req = requirements_cached(&s);
    Ok(paths
        .iter()
        .filter_map(|p| {
            for id in ["lbp1", "lbp2"] {
                if let Ok(r) = metadata::inspect(id, p, false, &req) {
                    if r.valid {
                        return Some(r);
                    }
                }
            }
            None
        })
        .collect())
}
#[tauri::command]
pub async fn check_release(engine: String) -> Result<release::Release> {
    tauri::async_runtime::spawn_blocking(move || release::latest(&engine))
        .await
        .map_err(err)?
}
#[tauri::command]
pub async fn refresh_requirements(app: tauri::AppHandle) -> Result<release::Requirements> {
    tauri::async_runtime::spawn_blocking(move || {
        let r = release::requirements()?;
        atomic_json(
            &app.state::<Store>().dir.join("requirements.json"),
            &r,
            false,
        )?;
        Ok(r)
    })
    .await
    .map_err(err)?
}
#[tauri::command]
pub fn install(app: tauri::AppHandle, engine: String) -> Result<()> {
    download::start(app, engine)
}
#[tauri::command]
pub fn cancel_download(s: tauri::State<Store>, engine: String) -> Result<()> {
    if let Some(t) = s.tasks.lock().unwrap().get(&engine) {
        t.cancel.store(true, Ordering::Relaxed)
    }
    Ok(())
}
#[tauri::command]
pub async fn maintenance(
    app: tauri::AppHandle,
    engine: String,
    action: String,
    target: Option<String>,
) -> Result<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let s = app.state::<Store>();
        idle(&s)?;
        {
            let mut busy = s.busy.lock().unwrap();
            if *busy {
                return Err("Another operation is active.".into());
            }
            *busy = true;
        }
        let result = download::maintenance(&s, &engine, &action, target.as_deref());
        *s.busy.lock().unwrap() = false;
        result
    })
    .await
    .map_err(err)?
}
pub fn launch_args(g: &Game, settings: bool) -> Result<Vec<String>> {
    if settings {
        return Ok(vec![]);
    }
    match g.engine.as_str() {
        "rpcs3" => {
            let boot =
                metadata::boot_path(Path::new(&g.data)).ok_or("No RPCS3 boot executable found.")?;
            let mut args = vec!["--no-gui".into()];
            if g.fullscreen {
                args.push("--fullscreen".into())
            }
            args.push(boot.to_string_lossy().into());
            Ok(args)
        }
        "lbppc" => Ok(vec!["--play".into()]),
        "lbprecomp" | "replanet" => Ok(vec![]),
        _ => Err("Unknown engine".into()),
    }
}
fn sync_lbppc(i: &Installation, g: &Game) -> Result<()> {
    let root = Path::new(&i.root);
    let path = root.join("user/launcher.json");
    download::no_links(&path)?;
    if path.exists() && fs::metadata(&path).map_err(err)?.len()>4*1024*1024 { return Err("LBPPC settings exceed 4 MB.".into()); }
    let ini=root.join("lbppc.ini");
    let ini_text=if g.video.enabled && g.video.frame_gen().is_some() {
        download::no_links(&ini)?;
        if ini.exists() && fs::metadata(&ini).map_err(err)?.len()>1024*1024 { return Err("LBPPC INI exceeds 1 MB.".into()); }
        let text=if ini.exists() { fs::read_to_string(&ini).map_err(|_|"LBPPC INI cannot be read; refusing to overwrite it.")? } else { String::new() };
        Some(crate::video::frame_gen_ini(&text,g.video.frame_gen().unwrap()))
    } else { None };
    let mut data = if path.exists() {
        serde_json::from_slice::<Value>(&fs::read(&path).map_err(err)?)
            .map_err(|_| "LBPPC settings cannot be read; refusing to overwrite them.")?
    } else {
        json!({})
    };
    let o = data
        .as_object_mut()
        .ok_or("Unexpected LBPPC settings format.")?;
    o.insert("game".into(), json!(g.data));
    o.insert("update".into(), json!(g.update));
    crate::video::patch_lbppc(&mut data,&g.video)?;
    atomic_json(&path, &data, true)?;
    if let Some(text)=ini_text { atomic_text(&ini,&text,true)?; }
    Ok(())
}
#[tauri::command]
pub async fn launch(app: tauri::AppHandle, id: String, settings: bool) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move||launch_inner(app,id,settings)).await.map_err(err)?
}
fn launch_inner(app: tauri::AppHandle, id: String, settings: bool) -> Result<()> {
    let s = app.state::<Store>();
    idle(&s)?;
    let c = s.get();
    let g = c
        .games
        .iter()
        .find(|g| g.id == id)
        .ok_or("Unknown game")?
        .clone();
    let i = c
        .installations
        .get(&g.engine)
        .ok_or("Install or import the engine first.")?;
    metadata::windows_exe(Path::new(&i.executable))?;
    if !settings {
        if g.engine=="rpcs3" && !crate::ps3::firmware_present(i)? { return Err("Choose Prepare RPCS3 before your first PS3 launch.".into()); }
        let check = validate(&s, &id)?;
        if !check.valid {
            return Err(check.messages.join("\n"));
        }
        if g.engine == "lbppc" && !g.video.native_test() {
            sync_lbppc(i, &g)?;
        }
    }
    let mut args = launch_args(&g, settings)?;
    if !settings && g.engine=="rpcs3" && (g.video.enabled||g.ps3_compatibility) {
        let profile=crate::video::rpcs3_profile(&s.dir,i,&g)?;
        args.splice(0..0,["--config".into(),profile.to_string_lossy().into()]);
    }
    if !settings && g.video.native_test() { args=crate::video::native_test_args(&s.dir,i,&g)?; }
    let native_test=!settings&&g.video.native_test();
    let mut child = Command::new(&i.executable)
        .args(args)
        .current_dir(&i.root)
        .spawn()
        .map_err(|e| format!("Windows could not start the engine: {e}"))?;
    s.running.lock().unwrap().push(id.clone());
    if !settings {
        s.change(|c| {
            game_mut(c, &id)?.last_played = now();
            activity(c, &format!("Started {}.", id.to_uppercase()));
            Ok(())
        })?;
    }
    std::thread::spawn(move || {
        let mut last = std::time::Instant::now();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let stopped = match child.try_wait() {
                Ok(Some(status)) => {
                    if native_test && !status.success() { let _=app.state::<Store>().change(|c|{activity(c,"Native rendering test stopped with an error. Open test logs and saves on the game page.");Ok(())}); }
                    true
                },
                Err(_) => true,
                _ => false,
            };
            if !settings && (last.elapsed().as_secs() >= 15 || stopped) {
                let delta = last.elapsed().as_secs();
                let _ = app.state::<Store>().change(|c| {
                    game_mut(c, &id)?.seconds += delta;
                    Ok(())
                });
                last = std::time::Instant::now();
            }
            if stopped {
                app.state::<Store>()
                    .running
                    .lock()
                    .unwrap()
                    .retain(|v| v != &id);
                let _ = app.emit("changed", ());
                break;
            }
        }
    });
    Ok(())
}
#[tauri::command]
pub fn open_location(s: tauri::State<Store>, kind: String, id: Option<String>) -> Result<()> {
    let c = s.get();
    let p = match kind.as_str() {
        "nativeTest" => {
            let g=c.games.iter().find(|g|Some(&g.id)==id.as_ref()).ok_or("Missing game")?;
            if g.id!="lbp3" || ![60,120].contains(&g.video.target_fps) { return Err("Choose a native test target first.".into()); }
            let path=s.dir.join("native-tests").join(g.video.target_fps.to_string());
            download::no_links(&path)?;fs::create_dir_all(&path).map_err(err)?;path
        },
        "config" => s.dir.clone(),
        "installRoot" => PathBuf::from(c.install_root),
        "engine" => PathBuf::from(
            &c.installations
                .get(id.as_deref().ok_or("Missing engine")?)
                .ok_or("Engine not installed")?
                .root,
        ),
        "data" => PathBuf::from(
            &c.games
                .iter()
                .find(|g| Some(&g.id) == id.as_ref())
                .ok_or("Missing game")?
                .data,
        ),
        _ => return Err("Unknown location".into()),
    };
    if !p.is_dir() {
        return Err("Folder is missing.".into());
    }
    Command::new("explorer.exe").arg(p).spawn().map_err(err)?;
    Ok(())
}
#[tauri::command]
pub fn open_link(key: String) -> Result<()> {
    let url = match key.as_str() {
        "lbppc" => "https://github.com/emurphy04/LBPPC",
        "lbppc-guide" => "https://lbppc.com/getting-started",
        "lbprecomp" => "https://github.com/benjidotsh/LittleBigRecomp",
        "replanet" => {
            "https://www.reddit.com/r/littlebigplanet/comments/1x9e57o/replanet_a_followup/"
        }
        "rpcs3" => "https://rpcs3.net/quickstart",
        "firmware" => "https://www.playstation.com/en-us/support/hardware/ps3/system-software/",
        "privacy" => "https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher/blob/main/PRIVACY.md",
        "project" => "https://github.com/YasrAX2/LittleBigPlanet-Unified-Launcher",
        _ => return Err("Unknown link".into()),
    };
    #[cfg(windows)]
    {
        Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .map_err(err)?;
    }
    Ok(())
}
#[tauri::command]
pub async fn backup_config(app: tauri::AppHandle) -> Result<String> {
    tauri::async_runtime::spawn_blocking(move||{let p=rfd::FileDialog::new().set_title("Save launcher configuration backup").set_file_name("Patchwork-Pod-backup.json").add_filter("JSON",&["json"]).save_file();if let Some(p)=p{atomic_json(&p,&app.state::<Store>().get(),false)?;Ok("Configuration backed up. It contains local paths; keep it private. Game saves are managed by the engine.".into())}else{Ok("Backup cancelled.".into())}}).await.map_err(err)?
}
#[tauri::command]
pub async fn restore_config(app: tauri::AppHandle) -> Result<String> {
    tauri::async_runtime::spawn_blocking(move||{let s=app.state::<Store>();idle(&s)?;let p=rfd::FileDialog::new().set_title("Restore launcher configuration").add_filter("JSON",&["json"]).pick_file();if let Some(p)=p{if fs::metadata(&p).map_err(err)?.len()>2*1024*1024{return Err("Backup exceeds size limit.".into())}let mut c:Config=serde_json::from_slice(&fs::read(p).map_err(err)?).map_err(err)?;migrate_appearance(&mut c);validate_config(&c)?;for i in c.installations.values_mut(){i.managed=false;i.rollback.clear();}c.background_artwork.clear();for g in &mut c.games{g.artwork.clear();}s.change(|current|{*current=c;Ok(())})?;Ok("Configuration restored. Installations imported as user-owned for safety. Revalidate game paths before playing.".into())}else{Ok("Restore cancelled.".into())}}).await.map_err(err)?
}
#[tauri::command]
pub fn import_art(s: tauri::State<Store>, id: String, path: Option<String>) -> Result<Config> {
    if !["lbp1", "lbp2", "lbp3", "background"].contains(&id.as_str()) {
        return Err("Unknown artwork target".into());
    }
    let name = if let Some(path) = path {
        let p = Path::new(&path);
        if fs::metadata(p).map_err(err)?.len() > 8 * 1024 * 1024 {
            return Err("Artwork must be smaller than 8 MB.".into());
        }
        let b = fs::read(p).map_err(err)?;
        let ext = if b.starts_with(b"\x89PNG\r\n\x1a\n") {
            "png"
        } else if b.starts_with(&[0xff, 0xd8, 0xff]) {
            "jpg"
        } else {
            return Err("Choose PNG or JPEG artwork.".into());
        };
        let name = format!("{id}.{ext}");
        fs::create_dir_all(s.dir.join("art")).map_err(err)?;
        fs::write(s.dir.join("art").join(&name), b).map_err(err)?;
        name
    } else {
        String::new()
    };
    s.change(|c| {
        if id == "background" { c.background_artwork = name; c.background = if c.background_artwork.is_empty() { "color".into() } else { "picture".into() }; } else { game_mut(c, &id)?.artwork = name; }
        Ok(())
    })
}
#[tauri::command]
pub async fn diagnostics(app: tauri::AppHandle) -> Result<String> {
    tauri::async_runtime::spawn_blocking(move||{let s=app.state::<Store>();let c=s.get();let report=json!({"launcher":env!("CARGO_PKG_VERSION"),"platform":"Windows x64","engines":c.installations.iter().map(|(key,i)|json!({"engine":key,"version":i.version,"managed":i.managed,"executableExists":Path::new(&i.executable).is_file()})).collect::<Vec<_>>(),"games":c.games.iter().map(|g|json!({"id":g.id,"engine":g.engine,"titleId":g.title_id,"version":g.version,"dataExists":Path::new(&g.data).is_dir()})).collect::<Vec<_>>(),"privacy":"No paths, logs, usernames, accounts, artwork or credentials included. Nothing uploaded."});if let Some(p)=rfd::FileDialog::new().set_file_name("Patchwork-Pod-diagnostics.json").save_file(){atomic_json(&p,&report,false)?;Ok("Private diagnostics saved locally. Nothing uploaded.".into())}else{Ok("Export cancelled.".into())}}).await.map_err(err)?
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_artwork_rejects_paths_and_oversize_files() {
        let dir=tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("art")).unwrap();
        fs::write(dir.path().join("art/background.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        assert!(local_artwork(dir.path(),"background.png","background",8).is_some());
        assert!(local_artwork(dir.path(),"background.png","background",7).is_none());
        assert!(local_artwork(dir.path(),"../background.png","background",8).is_none());
    }
    #[test]
    fn persistence_and_default_games() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::open(d.path().into()).unwrap();
        assert_eq!(s.get().games.len(), 3);
        s.change(|c| {
            c.theme = "night".into();
            Ok(())
        })
        .unwrap();
        assert_eq!(Store::open(d.path().into()).unwrap().get().theme, "night");
        assert!(s
            .change(|c| {
                c.schema = 999;
                Ok(())
            })
            .is_err());
    }
    #[test]
    fn launch_settings_no_guessed_flags() {
        let g = Game {
            engine: "lbppc".into(),
            ..Default::default()
        };
        assert_eq!(launch_args(&g, false).unwrap(), vec!["--play"]);
        assert!(launch_args(&g, true).unwrap().is_empty());
    }
    #[test]
    fn native_settings_preserve_accounts() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir(d.path().join("user")).unwrap();
        atomic_json(
            &d.path().join("user/launcher.json"),
            &json!({"password_protected":"opaque","resolution":"1920x1080","game":"old"}),
            false,
        )
        .unwrap();
        sync_lbppc(
            &Installation {
                root: d.path().to_string_lossy().into(),
                ..Default::default()
            },
            &Game {
                data: "new".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let v: Value =
            serde_json::from_slice(&fs::read(d.path().join("user/launcher.json")).unwrap())
                .unwrap();
        assert_eq!(v["password_protected"], "opaque");
        assert_eq!(v["resolution"], "1920x1080");
        assert_eq!(v["game"], "new");
    }
    #[test]
    fn native_video_sync_updates_both_files_and_keeps_private_fields() {
        let d=tempfile::tempdir().unwrap();
        fs::create_dir(d.path().join("user")).unwrap();
        atomic_json(&d.path().join("user/launcher.json"),&json!({"password_protected":"opaque","frame_gen":"off","controls":"keep"}),false).unwrap();
        fs::write(d.path().join("lbppc.ini"),"# original\nframe_gen=off\nserver=keep\n").unwrap();
        let i=Installation{root:d.path().to_string_lossy().into(),..Default::default()};
        let g=Game{engine:"lbppc".into(),video:crate::video::VideoSettings{enabled:true,target_fps:60,..Default::default()},..Default::default()};
        sync_lbppc(&i,&g).unwrap();
        let v:Value=serde_json::from_slice(&fs::read(d.path().join("user/launcher.json")).unwrap()).unwrap();
        assert_eq!(v["password_protected"],"opaque");assert_eq!(v["controls"],"keep");assert_eq!(v["frame_gen"],"fsr3");
        assert_eq!(fs::read_to_string(d.path().join("lbppc.ini")).unwrap(),"# original\nserver=keep\nframe_gen=fsr3\n");
        assert_eq!(fs::read_to_string(d.path().join("lbppc.backup.ini")).unwrap(),"# original\nframe_gen=off\nserver=keep\n");
    }

}

#[tauri::command]
pub async fn prepare_ps3(app: tauri::AppHandle, path: Option<String>) -> Result<String> {
    tauri::async_runtime::spawn_blocking(move||{
        let s=app.state::<Store>();idle(&s)?;
        { let mut busy=s.busy.lock().map_err(err)?;if *busy{return Err("Another setup is running.".into());}*busy=true; }
        let result=(||{
            let c=s.get();let i=c.installations.get("rpcs3").ok_or("Install or import RPCS3 first.")?;
            if let Some(path)=path {
                let path=Path::new(&path);download::no_links(path)?;
                if !path.extension().is_some_and(|v|v.eq_ignore_ascii_case("pup")) || fs::metadata(path).map_err(err)?.len()>512*1024*1024 { return Err("Choose an official PS3 firmware .PUP file under 512 MB.".into()); }
                let mut magic=[0;8];use std::io::Read;fs::File::open(path).map_err(err)?.read_exact(&mut magic).map_err(err)?;
                if &magic!=b"SCEUF\0\0\0"{return Err("Not a PS3 firmware PUP.".into());}
                crate::ps3::run_installer(i,"--installfw",path)?;
                if !crate::ps3::firmware_present(i)? { return Err("Firmware installation was not confirmed. Inspect RPCS3's log.".into()); }
            }else{crate::ps3::ensure_firmware(&s.dir,i,|message|{let _=app.emit("setup",message);},None)?;}
            Ok("RPCS3 firmware is ready. Add your LBP game folder or PS3 package.".into())
        })();*s.busy.lock().map_err(err)?=false;result
    }).await.map_err(err)?
}
#[tauri::command]
pub async fn import_ps3_package(app: tauri::AppHandle, id: String, path: String) -> Result<String> {
    tauri::async_runtime::spawn_blocking(move||{
        if !["lbp1","lbp2"].contains(&id.as_str()){return Err("Only LBP1/2 PS3 packages are supported here. LBP3 needs extracted PS4 folders.".into());}
        let s=app.state::<Store>();idle(&s)?;
        let title=crate::ps3::package_title(Path::new(&path))?;
        {let mut busy=s.busy.lock().map_err(err)?;if *busy{return Err("Another setup is running.".into());}*busy=true;}
        let result=(||{
            let c=s.get();let i=c.installations.get("rpcs3").ok_or("Install or import RPCS3 first.")?;
            crate::ps3::ensure_firmware(&s.dir,i,|message|{let _=app.emit("setup",message);},None)?;
            let _=app.emit("setup","Installing your PS3 package. Large packages can take several minutes...");
            let installed=crate::ps3::root(i)?.join("dev_hdd0/game").join(&title);
            let existed=installed.is_dir();
            crate::ps3::run_installer(i,"--installpkg",Path::new(&path))?;
            let current=c.games.iter().find(|g|g.id==id).ok_or("Unknown game")?;
            let r=crate::ps3::installed_game(current,&installed,&title,&requirements_cached(&s))?;
            s.change(|c|{let g=game_mut(c,&id)?;g.engine="rpcs3".into();g.data=r.path;g.title_id=r.title_id;g.version=r.version;g.region=r.region;Ok(())})?;
            Ok(if existed{"RPCS3 processed the package and the existing matching game is linked. Check RPCS3 for update/DLC details."}else{"Installed game folder found and linked. Some digital games still need your own game license in RPCS3."}.into())
        })();*s.busy.lock().map_err(err)?=false;let _=app.emit("changed",());result
    }).await.map_err(err)?
}

#[cfg(test)]
mod default_art_tests {
    use super::*;
    #[test]
    fn bundled_images_cover_all_games_background_and_palettes() {
        for id in ["lbp1","lbp2","lbp3","background"].iter().map(|s|s.to_string()).chain(PALETTE_IDS.iter().map(|s|format!("palette-{s}"))) {
            let uri=default_artwork(&id).unwrap();
            let bytes=base64::engine::general_purpose::STANDARD.decode(uri.strip_prefix("data:image/png;base64,").unwrap()).unwrap();
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }
        assert!(default_artwork("../config.json").is_none());
    }
    #[test]
    fn reset_look_preserves_game_data_and_accessibility() {
        let mut c=Config::default();
        c.theme="night".into();c.background="color".into();c.background_artwork="background.png".into();
        c.games[0].artwork="lbp1.png".into();c.games[0].data="C:\\Games\\LBP".into();
        c.games[0].favorite=true;c.text_scale=120;c.reduced_motion=true;
        let expected_path=c.games[0].data.clone();
        reset_look(&mut c);
        assert_eq!(c.background,"picture");assert_eq!(c.panel_opacity,55);
        assert!(c.background_artwork.is_empty());assert!(c.games[0].artwork.is_empty());
        assert_eq!(c.games[0].data,expected_path);assert!(c.games[0].favorite);
        assert_eq!(c.text_scale,120);assert!(c.reduced_motion);
    }
}
