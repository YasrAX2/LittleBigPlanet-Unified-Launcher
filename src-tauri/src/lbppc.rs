use crate::{download, metadata, model::*};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, io::{BufRead, BufReader, Write}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::{Arc, atomic::{AtomicBool, Ordering}}, time::{Duration, Instant}};
use tauri::{Emitter, Manager};

const BUILD_ID: &str = "lbppc-source";
fn hidden(command: &mut Command) {
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
}
fn reserve(s: &Store) -> Result<()> {
    let mut busy=s.busy.lock().map_err(err)?;
    if *busy || !s.running.lock().map_err(err)?.is_empty() { return Err("Close games/engine settings and finish other operations first.".into()); }
    *busy=true;Ok(())
}
fn installed(s: &Store) -> Result<Installation> {
    let i=s.get().installations.get("lbppc").cloned().ok_or("Install or build LBPPC first.")?;
    download::no_links(Path::new(&i.root))?;metadata::windows_exe(Path::new(&i.executable))?;Ok(i)
}
fn check_signin_helpers(root: &Path) -> Result<()> {
    for (name,expected) in [
        ("scripts/lbppc_signin.py","8cec0ad6a843b027536de5c984298025de0d201c0a8fc58319135ef69f13be33"),
        ("scripts/tls_trust.py","367ef8f8d43c505ff054209cb09ce90e0b074df3860403663b6938c34f52c3b1")
    ] {
        let path=root.join(name);download::no_links(&path)?;
        let bytes=fs::read(path).map_err(|_|"LBPPC sign-in helpers are missing. Repair the official 0.1.5 installation first.".to_string())?;
        let text=std::str::from_utf8(&bytes).map_err(err)?.replace("\r\n","\n");
        if format!("{:x}",Sha256::digest(text.as_bytes()))!=expected { return Err("This LBPPC sign-in helper version is not verified. Use the official LBPPC settings or install the supported 0.1.5 release.".into()); }
    }
    Ok(())
}
fn bridge(s: &Store, i: &Installation, action: &str) -> Result<Value> {
    check_signin_helpers(Path::new(&i.root))?;
    download::no_links(&Path::new(&i.root).join("user/launcher.json"))?;
    let helpers=s.dir.join("helpers");download::no_links(&helpers)?;fs::create_dir_all(&helpers).map_err(err)?;
    let script=helpers.join("lbppc_online.py");atomic_text(&script,include_str!("../experiments/lbppc_online.py"),false)?;
    let result=tempfile::NamedTempFile::new_in(&helpers).map_err(err)?;
    let cancel=s.dir.join("lbppc-signin.cancel");download::no_links(&cancel)?;
    let mut command=Command::new(&i.executable);
    command.args(["--script"]).arg(&script).arg("--root").arg(&i.root).args(["--action",action,"--result"]).arg(result.path()).arg("--cancel").arg(cancel)
        .current_dir(&i.root).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());hidden(&mut command);
    if action=="check" {
        let c=s.get();let g=c.games.iter().find(|g|g.id=="lbp3").unwrap();
        if g.engine!="lbppc" || g.data.is_empty() { return Err("Select the LBPPC / PS4 edition and its game folder first.".into()); }
        command.arg("--game").arg(&g.data);if !g.update.is_empty(){command.arg("--update").arg(&g.update);}
    }
    let mut child=command.spawn().map_err(err)?;let started=Instant::now();
    loop {
        if child.try_wait().map_err(err)?.is_some(){break;}
        if started.elapsed()>Duration::from_secs(330){let _=child.kill();let _=child.wait();return Err("LBPPC setup timed out. Retry or open the official LBPPC settings.".into());}
        std::thread::sleep(Duration::from_millis(150));
    }
    if fs::metadata(result.path()).map_err(err)?.len()>64*1024{return Err("Invalid LBPPC response size.".into());}
    let value:Value=serde_json::from_slice(&fs::read(result.path()).map_err(err)?).map_err(|_|"LBPPC did not return a setup result. Open its official settings for details.")?;
    if value.get("ok").and_then(Value::as_bool)!=Some(true){return Err(value.get("message").and_then(Value::as_str).unwrap_or("LBPPC setup did not finish.").into());}
    Ok(value)
}
#[tauri::command]
pub async fn lbppc_setup(app: tauri::AppHandle, action: String) -> Result<Value> {
    if !["status","signin","signout","check"].contains(&action.as_str()){return Err("Unknown LBPPC setup action.".into());}
    tauri::async_runtime::spawn_blocking(move||{
        let s=app.state::<Store>();let i=installed(&s)?;
        let write=action!="status";if write{reserve(&s)?;}
        let result=(||{
            if action=="signin" {
                let cancel=s.dir.join("lbppc-signin.cancel");download::no_links(&cancel)?;if cancel.exists(){fs::remove_file(cancel).map_err(err)?;}
                let _=app.emit("setup","Finish signing in or creating your LBPPC account in your browser. Passwords stay on the official website.");
            }
            let value=bridge(&s,&i,&action)?;
            if action=="check" && value.get("complete").and_then(Value::as_bool)==Some(true) {
                if let Some(path)=value.get("detectedUpdate").and_then(Value::as_str).filter(|v|!v.is_empty()) {
                    let req=crate::release::Requirements::default();
                    let r=metadata::inspect_for_engine("lbp3","lbppc",Path::new(path),true,&req)?;
                    let c=s.get();let g=c.games.iter().find(|g|g.id=="lbp3").unwrap();
                    if !r.valid || r.title_id!=g.title_id{return Err("Detected update does not match the selected PS4 game.".into());}
                    s.change(|c|{game_mut(c,"lbp3")?.update=r.path;Ok(())})?;
                }
                let c=s.get();let g=c.games.iter().find(|g|g.id=="lbp3").unwrap();crate::commands::sync_lbppc(&i,g)?;
            }
            Ok(value)
        })();
        if write{*s.busy.lock().unwrap()=false;}let _=app.emit("changed",());result
    }).await.map_err(err)?
}
#[tauri::command]
pub fn cancel_lbppc_signin(s: tauri::State<Store>) -> Result<()> {let path=s.dir.join("lbppc-signin.cancel");download::no_links(&path)?;atomic_text(&path,"cancel",false)}

fn update(app: &tauri::AppHandle, status: &str, error: &str) {
    let s=app.state::<Store>();let mut tasks=s.tasks.lock().unwrap();
    if let Some(task)=tasks.get_mut(BUILD_ID){task.status=status.into();task.error=error.into();let _=app.emit("download",task.clone());}
}
fn copy_user_data(from: &Path, to: &Path) -> Result<()> {
    let user=from.join("user");
    download::no_links(&user)?;
    if user.is_dir(){for file in download::files(&user)?{
        let relative=file.strip_prefix(&user).map_err(err)?;let target=to.join("user").join(relative);download::no_links(&target)?;
        fs::create_dir_all(target.parent().unwrap()).map_err(err)?;fs::copy(file,target).map_err(err)?;
    }}
    for name in ["lbppc.ini","input.ini"] {let file=from.join(name);if file.is_file(){download::no_links(&file)?;fs::copy(file,to.join(name)).map_err(err)?;}}
    Ok(())
}
fn build(app: &tauri::AppHandle, workspace: PathBuf, cancel: Arc<AtomicBool>) -> Result<()> {
    let s=app.state::<Store>();download::no_links(&workspace)?;fs::create_dir_all(&workspace).map_err(err)?;
    let log_dir=s.dir.join("builds");download::no_links(&log_dir)?;fs::create_dir_all(&log_dir).map_err(err)?;
    let script=log_dir.join("build_lbppc.ps1");atomic_text(&script,include_str!("../experiments/build_lbppc.ps1"),false)?;
    let c=s.get();let destination=Path::new(&c.install_root).join(format!("lbppc-source-{}",now()));download::no_links(&destination)?;
    let mut log=fs::File::create(log_dir.join("lbppc-build.log")).map_err(err)?;
    let system=std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(||PathBuf::from(r"C:\Windows")).join("System32");
    let shell=system.join("WindowsPowerShell/v1.0/powershell.exe");
    let mut command=Command::new(shell);
    command.args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script).arg("-Workspace").arg(&workspace).arg("-Destination").arg(&destination)
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::from(log.try_clone().map_err(err)?));hidden(&mut command);
    let mut child=command.spawn().map_err(err)?;let output=child.stdout.take().ok_or("Build output is unavailable.")?;
    let handle=app.clone();let reader=std::thread::spawn(move||{
        for line in BufReader::new(output).lines().map_while(std::result::Result::ok){
            let _=writeln!(log,"{line}");if let Some(status)=line.strip_prefix("PODBUILD: "){update(&handle,status,"");}
        }
    });
    let start=Instant::now();let status=loop{
        if let Some(status)=child.try_wait().map_err(err)?{break status;}
        if cancel.load(Ordering::Relaxed)||start.elapsed()>Duration::from_secs(7200){
            let mut stop=Command::new(system.join("taskkill.exe"));stop.args(["/PID",&child.id().to_string(),"/T","/F"]);hidden(&mut stop);let _=stop.status();let _=child.wait();let _=reader.join();
            return Err(if cancel.load(Ordering::Relaxed){"Source build cancelled. The existing engine is unchanged."}else{"Source build timed out. See the build log."}.into());
        }
        std::thread::sleep(Duration::from_millis(250));
    };let _=reader.join();
    if !status.success(){return Err("LBPPC source build failed. Open the build log for the exact step; the existing engine is unchanged.".into());}
    let executable=destination.join("LBPPC.exe");metadata::windows_exe(&executable)?;metadata::windows_exe(&destination.join("bin/LBPPC-Game.exe"))?;
    if let Some(previous)=c.installations.get("lbppc"){copy_user_data(Path::new(&previous.root),&destination)?;}
    let i=Installation{engine:"lbppc".into(),executable:executable.to_string_lossy().into(),root:destination.to_string_lossy().into(),version:"source-v0.1.5".into(),managed:false,rollback:String::new()};
    s.change(|c|{c.installations.insert("lbppc".into(),i);activity(c,"Built official LBPPC v0.1.5 from source. Existing engine files retained.");Ok(())})?;Ok(())
}
#[tauri::command]
pub fn build_lbppc(app: tauri::AppHandle, folder: Option<String>) -> Result<()> {
    let workspace=if let Some(folder)=folder{let p=PathBuf::from(folder);if !p.is_absolute()||!p.is_dir(){return Err("Choose an existing absolute build folder.".into());}p.join("PatchworkBuild")}else{
        PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("Windows local application folder is unavailable.")?).join("PatchworkBuild")
    };
    download::no_links(&workspace)?;
    let s=app.state::<Store>();reserve(&s)?;
    let cancel=Arc::new(AtomicBool::new(false));
    s.tasks.lock().unwrap().insert(BUILD_ID.into(),download::Task{engine:BUILD_ID.into(),version:"v0.1.5".into(),status:"Preparing source build".into(),downloaded:0,total:0,speed:0,error:String::new(),cancel:cancel.clone()});
    drop(s);
    std::thread::spawn(move||{
        let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||build(&app,workspace,cancel.clone()))).unwrap_or_else(|_|Err("Build stopped after an internal error; existing engine retained.".into()));
        match result {Ok(())=>update(&app,"Complete",""),Err(e)=>update(&app,if cancel.load(Ordering::Relaxed){"Cancelled"}else{"Failed"},&e)}
        *app.state::<Store>().busy.lock().unwrap()=false;let _=app.emit("changed",());
    });Ok(())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn copied_user_settings_keep_credentials_and_other_fields() {
        let d=tempfile::tempdir().unwrap();let source=d.path().join("old");let target=d.path().join("new");
        fs::create_dir_all(source.join("user/saves")).unwrap();fs::create_dir_all(&target).unwrap();
        fs::write(source.join("user/launcher.json"),br#"{"password_protected":"opaque","setting":"keep"}"#).unwrap();fs::write(source.join("user/saves/save.bin"),b"save").unwrap();
        copy_user_data(&source,&target).unwrap();assert_eq!(fs::read(source.join("user/launcher.json")).unwrap(),fs::read(target.join("user/launcher.json")).unwrap());assert!(target.join("user/saves/save.bin").is_file());
    }
    #[test] fn auth_helpers_are_pinned() {let d=tempfile::tempdir().unwrap();fs::create_dir(d.path().join("scripts")).unwrap();fs::write(d.path().join("scripts/lbppc_signin.py"),b"unexpected").unwrap();assert!(check_signin_helpers(d.path()).is_err());}
    #[test] #[ignore="reads the installed official LBPPC status locally; does not sign in or contact servers"]
    fn installed_official_status_bridge() {
        let d=tempfile::tempdir().unwrap();let s=Store::open(d.path().to_path_buf()).unwrap();
        let root=PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap()).join("com.patchworkpod.launcher/engines/lbppc");
        let i=Installation{engine:"lbppc".into(),root:root.to_string_lossy().into(),executable:root.join("LBPPC.exe").to_string_lossy().into(),version:"v0.1.5".into(),managed:false,rollback:String::new()};
        let value=bridge(&s,&i,"status").unwrap();assert!(value.get("signedIn").unwrap().is_boolean());
        assert!(value.get("refresh_token").is_none());assert!(value.get("access_token").is_none());
    }
}
