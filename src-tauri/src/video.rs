use crate::{download, model::{atomic_text, err, Game, Installation, Result}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::{Path, PathBuf}};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all="camelCase")]
pub struct VideoSettings {
    pub enabled: bool,
    pub resolution: String,
    pub display: String,
    pub target_fps: u32,
    pub frame_generation: String,
    pub upscaling: String,
    pub resolution_scale: u32,
    #[serde(skip_serializing_if="is_standard_rendering")]
    pub rendering: String,
}
fn is_standard_rendering(value: &str) -> bool { value=="standard" }
impl Default for VideoSettings {
    fn default() -> Self { Self {
        enabled: false, resolution: "1920x1080".into(), display: "borderless".into(),
        rendering: "standard".into(), target_fps: 0, frame_generation: "fsr3".into(), upscaling: "off".into(), resolution_scale: 100,
    } }
}
impl VideoSettings {
    pub fn validate(&self, engine: &str) -> Result<()> {
        if !["1280x720","1920x1080","2560x1440","3840x2160","2560x1080","3440x1440","3840x1600","3840x1080","4480x1260"].contains(&self.resolution.as_str())
            || !["windowed","borderless","fullscreen"].contains(&self.display.as_str())
            || ![0,30,60,120].contains(&self.target_fps)
            || !["fsr3","dlss"].contains(&self.frame_generation.as_str())
            || !["off","fsr1"].contains(&self.upscaling.as_str())
            || !["standard","native-test"].contains(&self.rendering.as_str())
            || ![100,150,200,300].contains(&self.resolution_scale) {
            return Err("Unsupported video setting.".into());
        }
        if self.enabled && engine=="lbppc" && self.rendering=="standard" && self.target_fps==120 {
            return Err("LBPPC currently supports 30 native FPS or 60 with frame generation. 120 FPS is not implemented by that engine.".into());
        }
        if self.enabled && self.rendering=="native-test" && (engine!="lbppc" || ![60,120].contains(&self.target_fps)) {
            return Err("Native rendering tests require LBPPC and a 60 or 120 FPS target.".into());
        }
        if self.enabled && !["rpcs3","lbppc"].contains(&engine) {
            return Err("Video overrides are available only for verified LBPPC and RPCS3 adapters.".into());
        }
        Ok(())
    }
    pub fn native_test(&self) -> bool { self.enabled && self.rendering=="native-test" }
    pub fn frame_gen(&self) -> Option<&str> {
        match self.target_fps { 30=>Some("off"),60=>Some(&self.frame_generation),_=>None }
    }
}

pub fn patch_lbppc(data: &mut Value, video: &VideoSettings) -> Result<()> {
    video.validate("lbppc")?;
    if !video.enabled || video.native_test() { return Ok(()); }
    let map=data.as_object_mut().ok_or("Invalid LBPPC settings document.")?;
    map.insert("resolution".into(),json!(video.resolution));
    map.insert("display".into(),json!(video.display));
    map.insert("upscaling".into(),json!(video.upscaling));
    if let Some(frame_gen)=video.frame_gen() { map.insert("frame_gen".into(),json!(frame_gen)); }
    Ok(())
}

pub fn frame_gen_ini(text: &str, mode: &str) -> String {
    let mut lines:Vec<_>=text.trim_start_matches('\u{feff}').lines().filter(|line| {
        line.split_once('=').map(|(key,_)|!key.trim().eq_ignore_ascii_case("frame_gen")).unwrap_or(true)
    }).map(str::to_string).collect();
    lines.push(format!("frame_gen={mode}"));
    lines.join("\n")+"\n"
}

fn read_yaml(path: &Path) -> Result<serde_yaml::Value> {
    download::no_links(path)?;
    if fs::metadata(path).map_err(err)?.len()>4*1024*1024 { return Err("RPCS3 configuration exceeds 4 MB.".into()); }
    let value:serde_yaml::Value=serde_yaml::from_slice(&fs::read(path).map_err(err)?).map_err(|_|"RPCS3 settings cannot be read; refusing to replace them.")?;
    if !value.is_mapping() { return Err("RPCS3 settings must be a YAML mapping.".into()); }
    Ok(value)
}

fn merge_yaml(base: &mut serde_yaml::Value, overlay: serde_yaml::Value) {
    if let (Some(base),Some(overlay))=(base.as_mapping_mut(),overlay.as_mapping()) {
        for (key,value) in overlay { merge_yaml(base.entry(key.clone()).or_insert(serde_yaml::Value::Null),value.clone()); }
    } else { *base=overlay; }
}

pub fn rpcs3_profile(dir: &Path, installation: &Installation, game: &Game) -> Result<PathBuf> {
    game.video.validate("rpcs3")?;
    if !["lbp1","lbp2","lbp3"].contains(&game.id.as_str()) { return Err("Unknown RPCS3 game profile.".into()); }
    let config_root=crate::ps3::root(installation)?;
    let engine_root=config_root.as_path();
    let root=engine_root;
    let title=game.title_id.as_bytes();
    let valid_title=title.len()==9 && title[..4].iter().all(u8::is_ascii_uppercase) && title[4..].iter().all(u8::is_ascii_digit);
    let custom=root.join("config/custom_configs").join(format!("config_{}.yml",game.title_id));
    let legacy_custom=root.join("custom_configs").join(format!("config_{}.yml",game.title_id));
    let global=[root.join("config/config.yml"),root.join("config.yml")].into_iter().find(|p|p.is_file());
    let mut data=match global { Some(path)=>read_yaml(&path)?, None=>serde_yaml::Value::Mapping(Default::default()) };
    let beside_game=crate::metadata::boot_path(Path::new(&game.data)).map(|p|p.with_extension("BIN.yml"));
    let custom=beside_game.filter(|p|p.is_file()).or_else(|| {
        if !valid_title { None } else if custom.is_file() { Some(custom) } else if legacy_custom.is_file() { Some(legacy_custom) } else { None }
    });
    let mut custom_color=false;
    if let Some(path)=custom { let overlay=read_yaml(&path)?;custom_color=overlay["Video"]["Write Color Buffers"].is_bool();merge_yaml(&mut data,overlay); }
    let mapping=data.as_mapping_mut().unwrap();
    let key=serde_yaml::Value::String("Video".into());
    let node=mapping.entry(key).or_insert_with(||serde_yaml::Value::Mapping(Default::default()));
    let video=node.as_mapping_mut().ok_or("Unexpected RPCS3 Video configuration.")?;
    if game.ps3_compatibility && !custom_color {video.insert(serde_yaml::Value::String("Write Color Buffers".into()),serde_yaml::Value::Bool(true));}
    if game.video.enabled {video.insert(serde_yaml::Value::String("Resolution Scale".into()),serde_yaml::to_value(game.video.resolution_scale).map_err(err)?); }
    if game.video.enabled && game.video.target_fps!=0 {
        video.insert(serde_yaml::Value::String("Frame limit".into()),serde_yaml::Value::String(game.video.target_fps.to_string()));
    }
    let output=dir.join("video").join(format!("rpcs3-{}.yml",game.id));
    download::no_links(&output)?;
    atomic_text(&output,&serde_yaml::to_string(&data).map_err(err)?,true)?;
    Ok(output)
}

// Official 0.1.5 release assets; unknown engines cannot run hard-coded research hooks.
pub fn native_test_args(dir: &Path, installation: &Installation, game: &Game) -> Result<Vec<String>> {
    game.video.validate(&game.engine)?;
    if !game.video.native_test() || game.id!="lbp3" || !["CUSA00473","CUSA00063"].contains(&game.title_id.as_str()) || !["1.28","01.28"].contains(&game.version.as_str()) {
        return Err("Native tests require a verified LBP3 US/Europe 1.28 game.".into());
    }
    let root=Path::new(&installation.root);
    if Path::new(&installation.executable).canonicalize().map_err(err)?!=root.join("LBPPC.exe").canonicalize().map_err(err)? { return Err("Import the official LBPPC.exe for native tests.".into()); }
    for (name,expected) in [
        ("LBPPC.exe","e3f2da1b3a4799fbec95784271c37a1aa2b1949845298d4ccf27795668c14d93"),
        ("run.py","441c699f3e1b05ca155d3ab88206f008fcf76083500fc421bf44fffb2cb58671"),
        ("bin/LBPPC-Game.exe","dfa36d62c1d440c03f81581e41af61bce3360c0bc75fd5a434d98fa562d2a8d1"),
    ] {
        let path=root.join(name); download::no_links(&path)?;
        if fs::metadata(&path).map_err(err)?.len()>64*1024*1024 || download::hash(&path)?!=expected {
            return Err("Native tests require the unmodified official LBPPC 0.1.5 Windows release. Other builds remain usable in Standard mode.".into());
        }
    }
    let test_dir=dir.join("native-tests").join(game.video.target_fps.to_string());
    download::no_links(&test_dir)?;
    fs::create_dir_all(&test_dir).map_err(err)?;
    let helper=dir.join("native-tests/lbppc_native.py");
    atomic_text(&helper,include_str!("../experiments/lbppc_native.py"),false)?;
    Ok(vec!["--script".into(),helper.to_string_lossy().into(),installation.root.clone(),test_dir.to_string_lossy().into(),game.video.target_fps.to_string(),game.data.clone(),game.update.clone(),game.video.resolution.clone(),game.video.display.clone(),game.video.upscaling.clone()])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_settings_keep_accounts_and_default_fps() {
        let mut data=json!({"password_protected":"opaque","frame_gen":"dlss","custom":42});
        let settings=VideoSettings{enabled:true,resolution:"2560x1440".into(),..Default::default()};
        patch_lbppc(&mut data,&settings).unwrap();
        assert_eq!(data["password_protected"],"opaque");assert_eq!(data["frame_gen"],"dlss");assert_eq!(data["custom"],42);
        assert_eq!(data["resolution"],"2560x1440");
        let settings=VideoSettings{enabled:true,target_fps:60,..Default::default()};
        patch_lbppc(&mut data,&settings).unwrap();assert_eq!(data["frame_gen"],"fsr3");
        assert!(VideoSettings{enabled:true,target_fps:120,..Default::default()}.validate("lbppc").is_err());
        assert!(VideoSettings{target_fps:999,..Default::default()}.validate("rpcs3").is_err());
    }
    #[test]
    fn ini_replaces_only_frame_generation() {
        let result=frame_gen_ini("# keep\nframe_gen=off\nsetting=yes\n frame_gen = dlss\n","fsr3");
        assert_eq!(result,"# keep\nsetting=yes\nframe_gen=fsr3\n");
    }
    #[test]
    fn rpc_profile_sets_120_without_changing_original_or_timing() {
        let root=tempfile::tempdir().unwrap();let dir=tempfile::tempdir().unwrap();
        let original="Core:\n  PPU Decoder: LLVM Recompiler\nVideo:\n  Frame limit: Auto\n  Vblank Rate: 60\n  Renderer: Vulkan\n";
        fs::write(root.path().join("config.yml"),original).unwrap();
        let game=Game{id:"lbp1".into(),video:VideoSettings{enabled:true,target_fps:120,resolution_scale:150,..Default::default()},..Default::default()};
        let installation=Installation{root:root.path().to_string_lossy().into(),..Default::default()};
        let path=rpcs3_profile(dir.path(),&installation,&game).unwrap();
        let generated:serde_yaml::Value=serde_yaml::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(generated["Video"]["Frame limit"].as_str(),Some("120"));
        assert_eq!(generated["Video"]["Vblank Rate"].as_u64(),Some(60));
        assert_eq!(generated["Video"]["Resolution Scale"].as_u64(),Some(150));
        assert_eq!(fs::read_to_string(root.path().join("config.yml")).unwrap(),original);
    }
    #[test]
    fn rpc_custom_profile_preserves_global_and_game_options() {
        let root=tempfile::tempdir().unwrap();let dir=tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("config/custom_configs")).unwrap();
        fs::write(root.path().join("config/config.yml"),"Video:\n  Renderer: Vulkan\n  Vblank Rate: 60\nCore:\n  SPU Block Size: Safe\n").unwrap();
        fs::write(root.path().join("config/custom_configs/config_BCUS98148.yml"),"Video:\n  Write Color Buffers: true\n").unwrap();
        let game=Game{title_id:"BCUS98148".into(),video:VideoSettings{enabled:true,target_fps:60,..Default::default()},..Default::default()};
        let installation=Installation{root:root.path().to_string_lossy().into(),..Default::default()};
        let path=rpcs3_profile(dir.path(),&installation,&game).unwrap();
        let generated:serde_yaml::Value=serde_yaml::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(generated["Video"]["Renderer"].as_str(),Some("Vulkan"));
        assert_eq!(generated["Video"]["Write Color Buffers"].as_bool(),Some(true));
        assert_eq!(generated["Core"]["SPU Block Size"].as_str(),Some("Safe"));
    }

    #[test]
    fn native_tests_are_opt_in_and_cannot_advertise_standard_120() {
        let v=VideoSettings{enabled:true,rendering:"native-test".into(),target_fps:120,..Default::default()};
        assert!(v.validate("lbppc").is_ok());assert!(v.native_test());assert!(v.validate("rpcs3").is_err());
        assert!(VideoSettings{target_fps:30,..v.clone()}.validate("lbppc").is_err());
        assert!(VideoSettings{rendering:"standard".into(),..v.clone()}.validate("lbppc").is_err());
        let original=json!({"password_protected":"private","frame_gen":"dlss"});let mut data=original.clone();
        patch_lbppc(&mut data,&v).unwrap();assert_eq!(data,original);
    }

    #[test]
    fn lbp_starting_profile_keeps_timing_scale_and_custom_choices() {
        let root=tempfile::tempdir().unwrap();let dir=tempfile::tempdir().unwrap();
        fs::write(root.path().join("config.yml"),"Video:\n  Frame limit: Auto\n  Resolution Scale: 200\n  Write Color Buffers: false\n  Vblank Rate: 60\nCore:\n  PPU Decoder: LLVM Recompiler\n").unwrap();
        let i=Installation{root:root.path().to_string_lossy().into(),..Default::default()};
        let g=Game{title_id:"NPUA80662".into(),..Default::default()};
        let path=rpcs3_profile(dir.path(),&i,&g).unwrap();
        let value=read_yaml(&path).unwrap();assert_eq!(value["Video"]["Resolution Scale"].as_u64(),Some(200));
        assert_eq!(value["Video"]["Frame limit"].as_str(),Some("Auto"));assert_eq!(value["Video"]["Write Color Buffers"].as_bool(),Some(true));
        fs::create_dir_all(root.path().join("config/custom_configs")).unwrap();
        fs::write(root.path().join("config/custom_configs/config_NPUA80662.yml"),"Video:\n  Write Color Buffers: false\n").unwrap();
        let path=rpcs3_profile(dir.path(),&i,&g).unwrap();assert_eq!(read_yaml(&path).unwrap()["Video"]["Write Color Buffers"].as_bool(),Some(false));
    }

}

#[cfg(test)] mod lbp3_profile_tests {
    use super::*;
    #[test] fn ps3_lbp3_has_its_own_profile_and_keeps_original() {
        let d=tempfile::tempdir().unwrap();let i=Installation{root:d.path().to_string_lossy().into(),..Default::default()};
        let g=Game{id:"lbp3".into(),engine:"rpcs3".into(),title_id:"BCUS98362".into(),..Default::default()};
        let path=rpcs3_profile(d.path(),&i,&g).unwrap();assert!(path.ends_with("rpcs3-lbp3.yml"));
        assert!(!d.path().join("config.yml").exists());
    }
}
