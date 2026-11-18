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
}
impl Default for VideoSettings {
    fn default() -> Self { Self {
        enabled: false, resolution: "1920x1080".into(), display: "borderless".into(),
        target_fps: 0, frame_generation: "fsr3".into(), upscaling: "off".into(), resolution_scale: 100,
    } }
}
impl VideoSettings {
    pub fn validate(&self, engine: &str) -> Result<()> {
        if !["1280x720","1920x1080","2560x1440","3840x2160","2560x1080","3440x1440","3840x1600","3840x1080","4480x1260"].contains(&self.resolution.as_str())
            || !["windowed","borderless","fullscreen"].contains(&self.display.as_str())
            || ![0,30,60,120].contains(&self.target_fps)
            || !["fsr3","dlss"].contains(&self.frame_generation.as_str())
            || !["off","fsr1"].contains(&self.upscaling.as_str())
            || ![100,150,200,300].contains(&self.resolution_scale) {
            return Err("Unsupported video setting.".into());
        }
        if self.enabled && engine=="lbppc" && self.target_fps==120 {
            return Err("LBPPC currently supports 30 native FPS or 60 with frame generation. 120 FPS is not implemented by that engine.".into());
        }
        if self.enabled && !["rpcs3","lbppc"].contains(&engine) {
            return Err("Video overrides are available only for verified LBPPC and RPCS3 adapters.".into());
        }
        Ok(())
    }
    pub fn frame_gen(&self) -> Option<&str> {
        match self.target_fps { 30=>Some("off"),60=>Some(&self.frame_generation),_=>None }
    }
}

pub fn patch_lbppc(data: &mut Value, video: &VideoSettings) -> Result<()> {
    video.validate("lbppc")?;
    if !video.enabled { return Ok(()); }
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
    let bytes=fs::read(path).map_err(err)?;
    let text=std::str::from_utf8(&bytes).map_err(|_|"RPCS3 settings are not valid UTF-8; original settings were left untouched.")?;
    // RPCS3 reads these as hexadecimal strings; YAML Value cannot store 128-bit integers.
    let normalized=text.lines().map(|line| {
        if let Some((key,value))=line.split_once(':') {
            let value=value.trim();
            let digits=value.strip_prefix("0x").or_else(||value.strip_prefix("0X")).unwrap_or(value);
            if ["Console PSID","Console IDPS"].contains(&key.trim()) && !digits.is_empty() && digits.len()<=32 && digits.bytes().all(|b|b.is_ascii_hexdigit()) {
                return format!("{key}: \"{value}\"");
            }
        }
        line.to_string()
    }).collect::<Vec<_>>().join("\n");
    let value:serde_yaml::Value=serde_yaml::from_str(&normalized).map_err(|e| {
        let line=e.location().map(|p|p.line()).unwrap_or(0);
        format!("RPCS3 settings cannot be read at line {line}; original settings were left untouched.")
    })?;
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
    if !["lbp1","lbp2"].contains(&game.id.as_str()) { return Err("Unknown RPCS3 game profile.".into()); }
    let engine_root=Path::new(&installation.root);
    let portable=engine_root.join("portable");
    // RPCS3 gives a portable folder precedence over its environment override.
    if !portable.is_dir() && std::env::var_os("RPCS3_CONFIG_DIR").is_some() {
        return Err("RPCS3_CONFIG_DIR is set. Keep video overrides off and configure video in RPCS3 to preserve that custom location.".into());
    }
    let root=if portable.is_dir() { portable.as_path() } else { engine_root };
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
    if let Some(path)=custom { merge_yaml(&mut data,read_yaml(&path)?); }
    let mapping=data.as_mapping_mut().unwrap();
    let key=serde_yaml::Value::String("Video".into());
    let node=mapping.entry(key).or_insert_with(||serde_yaml::Value::Mapping(Default::default()));
    let video=node.as_mapping_mut().ok_or("Unexpected RPCS3 Video configuration.")?;
    video.insert(serde_yaml::Value::String("Resolution Scale".into()),serde_yaml::to_value(game.video.resolution_scale).map_err(err)?);
    if game.video.target_fps!=0 {
        video.insert(serde_yaml::Value::String("Frame limit".into()),serde_yaml::Value::String(game.video.target_fps.to_string()));
    }
    let output=dir.join("video").join(format!("rpcs3-{}.yml",game.id));
    download::no_links(&output)?;
    atomic_text(&output,&serde_yaml::to_string(&data).map_err(err)?,true)?;
    Ok(output)
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

}

#[cfg(test)] mod rpc_yaml_tests {
    use super::*;
    #[test] fn console_identifiers_keep_exact_hex_digits_and_settings_stay_untouched() {
        let d=tempfile::tempdir().unwrap();let p=d.path().join("config.yml");
        let original="System:\n  Console PSID: 0x11112222333344445555666677778888\n  Console IDPS: 0x00000000000000000000000000000001\nVideo:\n  Frame limit: 30\n  Write Color Buffers: false\n";
        fs::write(&p,original).unwrap();let data=read_yaml(&p).unwrap();
        assert_eq!(data["System"]["Console PSID"].as_str(),Some("0x11112222333344445555666677778888"));
        assert_eq!(data["System"]["Console IDPS"].as_str(),Some("0x00000000000000000000000000000001"));
        assert_eq!(data["Video"]["Frame limit"].as_u64(),Some(30));
        assert_eq!(data["Video"]["Write Color Buffers"].as_bool(),Some(false));
        assert_eq!(fs::read_to_string(&p).unwrap(),original);
        let serialized=serde_yaml::to_string(&data).unwrap();
        assert_eq!(serde_yaml::from_str::<serde_yaml::Value>(&serialized).unwrap(),data);
    }
    #[test] fn invalid_yaml_is_still_rejected_without_echoing_private_values() {
        let d=tempfile::tempdir().unwrap();let p=d.path().join("config.yml");let original="Video: [\nprivate-secret: bad";
        fs::write(&p,original).unwrap();let error=read_yaml(&p).unwrap_err();
        assert!(!error.contains("private-secret"));assert_eq!(fs::read_to_string(&p).unwrap(),original);
    }
    #[test]
    #[ignore="reads the owner's local RPCS3 settings only when explicitly requested"]
    fn local_rpc_config_roundtrip() {
        let root=std::env::var("PATCHWORK_RPC_ROOT").unwrap();let root=Path::new(&root);
        let original=fs::read(root.join("config/config.yml")).unwrap();
        let d=tempfile::tempdir().unwrap();let i=Installation{root:root.to_string_lossy().into(),..Default::default()};
        let game=Game{id:"lbp1".into(),engine:"rpcs3".into(),..Default::default()};
        let profile=rpcs3_profile(d.path(),&i,&game).unwrap();read_yaml(&profile).unwrap();
        assert_eq!(fs::read(root.join("config/config.yml")).unwrap(),original);
    }
}
