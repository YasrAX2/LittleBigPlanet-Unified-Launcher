use crate::{download, metadata, model::{err, Installation, Result}};
use std::{fs, io::Read, path::{Path, PathBuf}, process::{Command, Stdio}, thread, time::{Duration, Instant, SystemTime}, sync::atomic::{AtomicBool,Ordering}};

pub fn root(i: &Installation) -> Result<PathBuf> {
    let root=Path::new(&i.root);
    if root.join("portable").is_dir() { return Ok(root.join("portable")); }
    if std::env::var_os("RPCS3_CONFIG_DIR").is_some() { return Err("A custom RPCS3_CONFIG_DIR is active. Use RPCS3 to manage its firmware/packages, then link the installed folder.".into()); }
    Ok(root.into())
}
pub fn firmware_present(i: &Installation) -> Result<bool> {
    let root=root(i)?;Ok(!root.join(".patchwork-firmware-incomplete").exists() && root.join("dev_flash/vsh/module/vsh.self").is_file())
}
pub fn ensure_firmware(dir: &Path, i: &Installation, progress: impl Fn(&str), cancel: Option<&AtomicBool>) -> Result<()> {
    if firmware_present(i)? { return Ok(()); }
    const HASH: &str="158471fd834f8ea8036136b6aab43cd86c7ba73d79ca30e0af3c0fe0001cf365";
    const SIZE:u64=206197916;
    const URL:&str="http://dus01.ps3.update.playstation.net/update/ps3/image/us/2026_0318_a2b60b6ac1d2e49e230144345616927c/PS3UPDAT.PUP";
    let cache=dir.join("downloads/PS3UPDAT-4.93.PUP"); download::no_links(&cache)?;
    if !cache.is_file() || fs::metadata(&cache).map_err(err)?.len()!=SIZE || download::hash(&cache)?!=HASH {
        progress("Downloading and verifying official PS3 firmware...");
        let client=reqwest::blocking::Client::builder().timeout(Duration::from_secs(300)).connect_timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none()).build().map_err(err)?;
        // Sony publishes this HTTP URL on its HTTPS support page. The pinned
        // SHA-256 authenticates the exact file before RPCS3's own PUP checks.
        let response=client.get(URL).send().map_err(err)?.error_for_status().map_err(err)?;
        if response.content_length().is_some_and(|n|n!=SIZE) { return Err("Official firmware size changed. Use Install firmware file or review the launcher update.".into()); }
        let parent=cache.parent().unwrap();fs::create_dir_all(parent).map_err(err)?;
        let mut temp=tempfile::NamedTempFile::new_in(parent).map_err(err)?;
        let mut response=response.take(SIZE+1);let mut copied=0u64;let mut buffer=[0u8;65536];
        loop { if cancel.is_some_and(|v|v.load(Ordering::Relaxed)){return Err("Firmware download cancelled.".into());}let n=response.read(&mut buffer).map_err(err)?;if n==0{break;}use std::io::Write;temp.write_all(&buffer[..n]).map_err(err)?;copied+=n as u64; }
        if copied!=SIZE || download::hash(temp.path())?!=HASH { return Err("PS3 firmware checksum mismatch; nothing was installed.".into()); }
        temp.as_file().sync_all().map_err(err)?;temp.persist(&cache).map_err(err)?;
    }
    progress("Installing PS3 firmware in RPCS3...");
    run_installer_cancellable(i,"--installfw",&cache,cancel)?;
    if !firmware_present(i)? { return Err("RPCS3 did not create a usable firmware installation. Use Install firmware file or inspect RPCS3's log.".into()); }
    Ok(())
}
pub fn package_title(path: &Path) -> Result<String> {
    download::no_links(path)?;
    if !path.extension().is_some_and(|v|v.eq_ignore_ascii_case("pkg")) { return Err("Choose a PS3 .pkg file.".into()); }
    let size=fs::metadata(path).map_err(err)?.len();
    if !(128..=64*1024*1024*1024).contains(&size) { return Err("Package size is unsupported.".into()); }
    let mut data=[0u8;128]; fs::File::open(path).map_err(err)?.read_exact(&mut data).map_err(err)?;
    if &data[..4]!=b"\x7fPKG" || data[6..8]!=[0,1] { return Err("This is not a PS3 package. PS4 packages cannot be installed by RPCS3.".into()); }
    if u64::from_be_bytes(data[24..32].try_into().unwrap())!=size { return Err("Package is incomplete or has an invalid size.".into()); }
    let title=&data[55..64];
    if !title[..4].iter().all(u8::is_ascii_uppercase)||!title[4..].iter().all(u8::is_ascii_digit) { return Err("Package content ID is invalid.".into()); }
    Ok(String::from_utf8(title.to_vec()).map_err(err)?)
}
pub fn installed_game(current: &crate::model::Game, installed: &Path, title: &str, req: &crate::release::Requirements) -> Result<metadata::Inspection> {
    let data=metadata::sfo_at(installed).map_err(|_|"No matching installed game metadata was found. Updates/DLC require their base game.".to_string())?;
    if current.title_id==title && !current.data.is_empty() {
        if let Ok(mut r)=metadata::inspect_for_engine(&current.id,"rpcs3",Path::new(&current.data),false,req) {
            if r.valid && r.title_id==title {
                if let Some(version)=data.get("APP_VER"){r.version=version.clone();}
                return Ok(r);
            }
        }
    }
    if data.get("CATEGORY").is_some_and(|v|v=="GD") {return Err("Update installed. Link the original base-game folder before playing; an update is not a complete game.".into());}
    let r=metadata::inspect_for_engine(&current.id,"rpcs3",installed,false,req)?;
    if !r.valid || r.title_id!=title {return Err("Installed content is not a complete matching LBP game. Check RPCS3; original package retained.".into());}
    Ok(r)
}
fn installation_confirmed(i: &Installation, option: &str, source: &Path, started: SystemTime) -> Result<bool> {
    use std::io::{Seek,SeekFrom};
    let root=root(i)?;
    let candidates=[root.join("log/RPCS3.log"),root.join("RPCS3.log")];
    for log in candidates {
        if !log.is_file(){continue;}download::no_links(&log)?;
        let meta=fs::metadata(&log).map_err(err)?;
        if meta.modified().map_err(err)?<started{continue;}
        let mut file=fs::File::open(log).map_err(err)?;file.seek(SeekFrom::Start(meta.len().saturating_sub(2*1024*1024))).map_err(err)?;
        let mut bytes=Vec::new();file.take(2*1024*1024).read_to_end(&mut bytes).map_err(err)?;let text=String::from_utf8_lossy(&bytes);
        if option=="--installfw" {
            if text.contains("Successfully installed PS3 firmware version ") && root.join("dev_flash/vsh/module/vsh.self").is_file(){return Ok(true);}
        }else{
            let title=package_title(source)?;
            if text.lines().any(|line|line.contains("Successfully installed ") && line.contains(&format!("title_id={title},"))){return Ok(true);}
        }
    }
    Ok(false)
}
pub fn run_installer(i: &Installation, option: &str, path: &Path) -> Result<()> {run_installer_cancellable(i,option,path,None)}
fn run_installer_cancellable(i: &Installation, option: &str, path: &Path, cancel: Option<&AtomicBool>) -> Result<()> {
    if !["--installpkg","--installfw"].contains(&option) { return Err("Unknown PS3 installation action.".into()); }
    if cancel.is_some_and(|v|v.load(Ordering::Relaxed)){return Err("Firmware setup cancelled.".into());}
    metadata::windows_exe(Path::new(&i.executable))?;
    let marker=root(i)?.join(".patchwork-firmware-incomplete");
    if option=="--installfw" { crate::model::atomic_text(&marker,"Firmware installation has not completed. Retry Prepare RPCS3.\n",false)?; }
    let mut command=Command::new(&i.executable);
    let console=root(i)?.join("log/patchwork-installer-console.log");download::no_links(&console)?;
    fs::create_dir_all(console.parent().unwrap()).map_err(err)?;let output=fs::File::create(console).map_err(err)?;
    command.args(["--headless",option]).arg(path).current_dir(&i.root).stdin(Stdio::null()).stdout(Stdio::from(output.try_clone().map_err(err)?)).stderr(Stdio::from(output));
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    let started=SystemTime::now();let mut child=command.spawn().map_err(err)?; let start=Instant::now();
    loop {
        if let Some(status)=child.try_wait().map_err(err)? { return if installation_confirmed(i,option,path,started)?{
            if option=="--installfw" {
                if !root(i)?.join("dev_flash/vsh/module/vsh.self").is_file(){return Err("Firmware installation did not produce its required files.".into());}
                download::no_links(&marker)?;fs::remove_file(marker).map_err(err)?;
            }
            crate::model::atomic_json(&root(i)?.join("patchwork-setup-result.json"),&serde_json::json!({"operation":option,"installationConfirmed":true,"shutdownExitCode":status.code(),"shutdownAbnormal":!status.success()}),false)?;
            Ok(())
        }else{Err("RPCS3 installation was not confirmed. Check RPCS3's local log; existing game data was retained.".into())}; }
        if cancel.is_some_and(|v|v.load(Ordering::Relaxed)){let _=child.kill();let _=child.wait();return Err("Firmware setup cancelled. Retry Prepare RPCS3 before playing.".into());}
        if start.elapsed()>Duration::from_secs(1800) { let _=child.kill();let _=child.wait();return Err("RPCS3 setup timed out. Inspect its log before retrying.".into()); }
        thread::sleep(Duration::from_millis(300));
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn package_headers_reject_ps4_truncation_and_bad_ids() {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("example.pkg");
        let mut data=[0u8;128];data[..4].copy_from_slice(b"\x7fPKG");data[6..8].copy_from_slice(&[0,1]);
        data[24..32].copy_from_slice(&128u64.to_be_bytes());data[55..64].copy_from_slice(b"NPUA80662");
        fs::write(&path,data).unwrap();assert_eq!(package_title(&path).unwrap(),"NPUA80662");
        data[0..4].copy_from_slice(b"\x7fCNT");fs::write(&path,data).unwrap();assert!(package_title(&path).is_err());
        data[..4].copy_from_slice(b"\x7fPKG");data[24..32].copy_from_slice(&256u64.to_be_bytes());fs::write(&path,data).unwrap();assert!(package_title(&path).is_err());
        data[24..32].copy_from_slice(&128u64.to_be_bytes());data[55]=b'/';fs::write(&path,data).unwrap();assert!(package_title(&path).is_err());
    }
    #[test] fn detects_portable_firmware() {
        let dir=tempfile::tempdir().unwrap();let i=Installation{root:dir.path().to_string_lossy().into(),..Default::default()};
        fs::create_dir_all(dir.path().join("portable/dev_flash/vsh/module")).unwrap();
        assert!(!firmware_present(&i).unwrap());fs::write(dir.path().join("portable/dev_flash/vsh/module/vsh.self"),b"fixture").unwrap();assert!(firmware_present(&i).unwrap());
    }
    #[test]
    #[ignore = "requires isolated official RPCS3 folder and network firmware download"]
    fn official_firmware_setup_smoke() {
        let root=std::env::var("PATCHWORK_RPC_TEST_ROOT").expect("isolated test root");
        let dir=tempfile::tempdir().unwrap();
        if let Some(source)=std::env::var_os("PATCHWORK_FIRMWARE_TEST_FILE") { fs::create_dir_all(dir.path().join("downloads")).unwrap();fs::copy(source,dir.path().join("downloads/PS3UPDAT-4.93.PUP")).unwrap(); }
        let i=Installation{root:root.clone(),executable:Path::new(&root).join("rpcs3.exe").to_string_lossy().into(),..Default::default()};
        crate::model::atomic_text(&Path::new(&root).join(".patchwork-firmware-incomplete"),"Isolated integration test reinstall",false).unwrap();
        ensure_firmware(dir.path(),&i,|s|println!("{s}"),None).unwrap();
        assert!(firmware_present(&i).unwrap());
        let bad=dir.path().join("invalid.pkg");let mut bytes=[0u8;128];bytes[..4].copy_from_slice(b"\x7fPKG");bytes[6..8].copy_from_slice(&[0,1]);bytes[24..32].copy_from_slice(&128u64.to_be_bytes());bytes[55..64].copy_from_slice(b"NPUA80662");fs::write(&bad,bytes).unwrap();
        assert!(run_installer(&i,"--installpkg",&bad).is_err());
        let installed=Path::new(&root).join("dev_hdd0/game/NPUA80662");
        assert!(!installed.join("USRDIR/EBOOT.BIN").exists());
        assert!(installed_game(&crate::model::Game{id:"lbp2".into(),..Default::default()},&installed,"NPUA80662",&Default::default()).is_err());
    }

    #[test] fn completion_requires_fresh_explicit_evidence() {
        let dir=tempfile::tempdir().unwrap();let root=dir.path();
        let i=Installation{root:root.to_string_lossy().into(),..Default::default()};
        fs::create_dir_all(root.join("log")).unwrap();fs::create_dir_all(root.join("dev_flash/vsh/module")).unwrap();
        fs::write(root.join("dev_flash/vsh/module/vsh.self"),b"fixture").unwrap();
        fs::write(root.join("log/RPCS3.log"),"No completion record").unwrap();
        assert!(!installation_confirmed(&i,"--installfw",Path::new("unused"),SystemTime::UNIX_EPOCH).unwrap());
        fs::write(root.join("log/RPCS3.log"),"Successfully installed PS3 firmware version 4.93.").unwrap();
        assert!(installation_confirmed(&i,"--installfw",Path::new("unused"),SystemTime::UNIX_EPOCH).unwrap());
        assert!(!installation_confirmed(&i,"--installfw",Path::new("unused"),SystemTime::now()+Duration::from_secs(10)).unwrap());
    }

    #[test] fn updates_preserve_the_linked_disc_folder() {
        let temp=tempfile::tempdir().unwrap();let disc=temp.path().join("disc");let update=temp.path().join("update");
        fs::create_dir_all(disc.join("PS3_GAME/USRDIR")).unwrap();fs::create_dir_all(update.join("USRDIR")).unwrap();
        fs::write(disc.join("PS3_GAME/PARAM.SFO"),metadata::fixture("LittleBigPlanet 2","BCUS98245","01.00")).unwrap();
        fs::write(disc.join("PS3_GAME/USRDIR/EBOOT.BIN"),b"original").unwrap();
        fs::write(update.join("PARAM.SFO"),metadata::fixture("LittleBigPlanet 2","BCUS98245","01.33")).unwrap();
        fs::write(update.join("USRDIR/EBOOT.BIN"),b"update").unwrap();
        let current=crate::model::Game{id:"lbp2".into(),title_id:"BCUS98245".into(),data:disc.to_string_lossy().into(),..Default::default()};
        let r=installed_game(&current,&update,"BCUS98245",&Default::default()).unwrap();
        assert_eq!(Path::new(&r.path).canonicalize().unwrap(),disc.canonicalize().unwrap());assert_eq!(r.version,"01.33");
        assert_eq!(fs::read(disc.join("PS3_GAME/USRDIR/EBOOT.BIN")).unwrap(),b"original");
    }

}

#[cfg(test)] mod lbp3_tests {
    use super::*;
    #[test] fn installed_lbp3_ps3_package_is_linkable_without_ps4_files() {
        let d=tempfile::tempdir().unwrap();fs::create_dir(d.path().join("USRDIR")).unwrap();
        fs::write(d.path().join("USRDIR/EBOOT.BIN"),b"test").unwrap();
        fs::write(d.path().join("PARAM.SFO"),metadata::fixture("LittleBigPlanet 3","NPUA81116","01.00")).unwrap();
        let g=crate::model::Game{id:"lbp3".into(),engine:"rpcs3".into(),..Default::default()};
        assert!(installed_game(&g,d.path(),"NPUA81116",&Default::default()).unwrap().valid);
        assert!(installed_game(&g,d.path(),"NPUA99999",&Default::default()).is_err());
    }
}
