use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
pub type Result<T> = std::result::Result<T, String>;
pub fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub engine: String,
    pub data: String,
    pub update: String,
    pub title_id: String,
    pub region: String,
    pub version: String,
    pub favorite: bool,
    pub seconds: u64,
    pub last_played: u64,
    pub fullscreen: bool,
    pub artwork: String,
    pub video: crate::video::VideoSettings,
}
impl Default for Game {
    fn default() -> Self {
        Self {
            id: "lbp1".into(),
            engine: "rpcs3".into(),
            data: String::new(),
            update: String::new(),
            title_id: String::new(),
            region: String::new(),
            version: String::new(),
            favorite: false,
            seconds: 0,
            last_played: 0,
            fullscreen: false,
            artwork: String::new(),
            video: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Installation {
    pub engine: String,
    pub executable: String,
    pub version: String,
    pub managed: bool,
    pub root: String,
    pub rollback: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub schema: u32,
    pub theme: String,
    pub palette: String,
    pub background: String,
    pub background_blur: u32,
    pub background_artwork: String,
    pub panel_opacity: u32,
    pub reduced_motion: bool,
    pub text_scale: u32,
    pub controller: bool,
    pub onboarding: bool,
    pub install_root: String,
    pub games: Vec<Game>,
    pub installations: BTreeMap<String, Installation>,
    pub activity: Vec<String>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            schema: 1,
            theme: "day".into(),
            palette: "black-green".into(),
            background: "picture".into(),
            background_blur: 8,
            background_artwork: String::new(),
            panel_opacity: 55,
            reduced_motion: false,
            text_scale: 100,
            controller: true,
            onboarding: false,
            install_root: String::new(),
            games: (1..=3)
                .map(|n| Game {
                    id: format!("lbp{n}"),
                    engine: if n == 3 {
                        "lbppc".into()
                    } else {
                        "rpcs3".into()
                    },
                    ..Default::default()
                })
                .collect(),
            installations: BTreeMap::new(),
            activity: vec![],
        }
    }
}
pub struct Store {
    pub dir: PathBuf,
    pub config: Mutex<Config>,
    pub busy: Mutex<bool>,
    pub running: Mutex<Vec<String>>,
    pub tasks: Mutex<BTreeMap<String, crate::download::Task>>,
}
impl Store {
    pub fn open(dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&dir).map_err(err)?;
        let path = dir.join("config.json");
        let mut c: Config = if path.exists() {
            serde_json::from_slice(&fs::read(path).map_err(err)?).map_err(|_| {
                "Configuration is unreadable. Restore config.backup.json before starting."
                    .to_string()
            })?
        } else {
            Config::default()
        };
        migrate_appearance(&mut c);
        validate_config(&c)?;
        if c.install_root.is_empty() {
            c.install_root = dir.join("engines").to_string_lossy().into()
        };
        Ok(Self {
            dir,
            config: Mutex::new(c),
            busy: Mutex::new(false),
            running: Mutex::new(vec![]),
            tasks: Mutex::new(BTreeMap::new()),
        })
    }
    pub fn change(&self, f: impl FnOnce(&mut Config) -> Result<()>) -> Result<Config> {
        let mut guard = self.config.lock().map_err(err)?;
        let mut c = guard.clone();
        f(&mut c)?;
        validate_config(&c)?;
        atomic_json(&self.dir.join("config.json"), &c, true)?;
        *guard = c.clone();
        Ok(c)
    }
    pub fn get(&self) -> Config {
        self.config.lock().unwrap().clone()
    }
}
pub fn migrate_appearance(c: &mut Config) {
    for g in &mut c.games { if g.id=="lbp1" && g.engine=="lbprecomp" { g.engine="rpcs3".into();g.video=Default::default(); } }
    if ["cool-island", "ocean", "grape", "berry", "sunshine", "sand", "mint", "midnight"].contains(&c.palette.as_str()) {
        c.palette = "black-green".into();
    }
}
pub const PALETTE_IDS: [&str; 10] = ["black-green", "red-burgundy", "purple-black", "pink-cream", "blue-grey", "olive-maroon", "amber-mint", "violet-green", "peach-lavender", "cyan-yellow"];
pub fn valid_artwork(name: &str, id: &str) -> bool {
    name.is_empty() || name == format!("{id}.png") || name == format!("{id}.jpg")
}
pub fn validate_config(c: &Config) -> Result<()> {
    if c.schema != 1
        || c.games.len() != 3
        || !["day", "night", "system"].contains(&c.theme.as_str())
        || !["picture", "color"].contains(&c.background.as_str())
        || c.background_blur > 24
        || !valid_artwork(&c.background_artwork, "background")
        || !(85..=150).contains(&c.text_scale)
        || !(55..=95).contains(&c.panel_opacity)
        || !PALETTE_IDS.contains(&c.palette.as_str())
    {
        return Err("Unsupported configuration.".into());
    }
    for id in ["lbp1", "lbp2", "lbp3"] {
        if c.games.iter().filter(|g| g.id == id).count() != 1 {
            return Err("Each game must appear exactly once.".into());
        }
    }
    for g in &c.games {
        g.video.validate(&g.engine)?;
        if !valid_artwork(&g.artwork, &g.id) { return Err("Invalid local artwork name.".into()); }
        if !["lbppc", "rpcs3", "lbprecomp", "replanet"].contains(&g.engine.as_str()) {
            return Err("Unknown game engine.".into());
        }
    }
    Ok(())
}
pub fn atomic_json(path: &Path, value: &impl Serialize, backup: bool) -> Result<()> {
    crate::download::no_links(path)?;
    crate::download::no_links(&path.with_extension("backup.json"))?;
    let parent = path.parent().ok_or("Invalid configuration path")?;
    fs::create_dir_all(parent).map_err(err)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(err)?;
    serde_json::to_writer_pretty(&mut tmp, value).map_err(err)?;
    tmp.as_file().sync_all().map_err(err)?;
    if backup && path.is_file() {
        fs::copy(path, path.with_extension("backup.json")).map_err(err)?;
    }
    tmp.persist(path).map_err(err)?;
    Ok(())
}
pub fn atomic_text(path: &Path, text: &str, backup: bool) -> Result<()> {
    use std::io::Write;
    crate::download::no_links(path)?;
    let parent=path.parent().ok_or("Invalid settings path")?;
    fs::create_dir_all(parent).map_err(err)?;
    let mut temp=tempfile::NamedTempFile::new_in(parent).map_err(err)?;
    temp.write_all(text.as_bytes()).map_err(err)?;
    temp.as_file().sync_all().map_err(err)?;
    if backup && path.is_file() {
        let extension=format!("backup.{}",path.extension().and_then(|v|v.to_str()).unwrap_or("txt"));
        let backup_path=path.with_extension(extension);
        crate::download::no_links(&backup_path)?;
        fs::copy(path,backup_path).map_err(err)?;
    }
    temp.persist(path).map_err(err)?;
    Ok(())
}
pub fn activity(c: &mut Config, text: &str) {
    c.activity.insert(0, text.into());
    c.activity.truncate(24)
}
pub fn game_mut<'a>(c: &'a mut Config, id: &str) -> Result<&'a mut Game> {
    c.games
        .iter_mut()
        .find(|g| g.id == id)
        .ok_or("Unknown game".into())
}

#[cfg(test)]
mod appearance_tests {
    use super::*;
    #[test]
    fn legacy_palette_migrates_without_changing_game_paths() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.palette = "grape".into();
        config.games[0].data = "C:\\Games\\LBP".into();
        atomic_json(&temp.path().join("config.json"), &config, false).unwrap();
        let saved = Store::open(temp.path().into()).unwrap().get();
        assert_eq!(saved.palette, "black-green");
        assert_eq!(saved.games[0].data, config.games[0].data);
    }
    #[test]
    fn legacy_configuration_gets_appearance_defaults() {
        let mut json=serde_json::to_value(Config::default()).unwrap();
        json.as_object_mut().unwrap().remove("palette");
        json.as_object_mut().unwrap().remove("panelOpacity");
        json.as_object_mut().unwrap().remove("background");
        json.as_object_mut().unwrap().remove("backgroundBlur");
        json.as_object_mut().unwrap().remove("backgroundArtwork");
        let old:Config=serde_json::from_value(json).unwrap();
        assert_eq!(old.palette,"black-green");
        assert_eq!(old.panel_opacity,55);
        assert_eq!(old.background,"picture");
        assert_eq!(old.background_blur,8);
        assert!(old.background_artwork.is_empty());
        validate_config(&old).unwrap();
    }
    #[test]
    fn appearance_persists_and_rejects_invalid_values() {
        let dir=tempfile::tempdir().unwrap();
        let store=Store::open(dir.path().into()).unwrap();
        store.change(|c|{c.palette="purple-black".into();c.panel_opacity=65;c.background="color".into();c.background_blur=12;Ok(())}).unwrap();
        let saved=Store::open(dir.path().into()).unwrap().get();
        assert_eq!((saved.palette.as_str(),saved.panel_opacity),("purple-black",65));
        assert_eq!(saved.background,"color");
        assert_eq!(saved.background_blur,12);
        assert!(store.change(|c|{c.background_blur=25;Ok(())}).is_err());
        assert!(store.change(|c|{c.background_artwork="../private.png".into();Ok(())}).is_err());
        assert!(store.change(|c|{c.background="https://invalid.example/image".into();Ok(())}).is_err());
        assert!(store.change(|c|{c.palette="invalid".into();Ok(())}).is_err());
        assert!(store.change(|c|{c.panel_opacity=0;Ok(())}).is_err());
        assert_eq!(store.get().palette,"purple-black");
        assert_eq!(store.get().panel_opacity,65);
        assert_eq!(store.get().background,"color");
        assert_eq!(store.get().background_blur,12);
        store.change(|c|{c.background="picture".into();Ok(())}).unwrap();
        assert_eq!(Store::open(dir.path().into()).unwrap().get().background,"picture");
    }
    #[test]
    fn legacy_video_defaults_and_invalid_video_do_not_corrupt_store() {
        let temp=tempfile::tempdir().unwrap();
        let game:Game=serde_json::from_value(serde_json::json!({"id":"lbp1"})).unwrap();
        assert!(!game.video.enabled);assert_eq!(game.video.target_fps,0);
        let store=Store::open(temp.path().into()).unwrap();
        store.change(|c|{c.games[0].video.enabled=true;c.games[0].video.target_fps=120;Ok(())}).unwrap();
        assert!(store.change(|c|{c.games[2].video.enabled=true;c.games[2].video.target_fps=120;Ok(())}).is_err());
        let saved=Store::open(temp.path().into()).unwrap().get();
        assert_eq!(saved.games[0].video.target_fps,120);assert!(!saved.games[2].video.enabled);
    }

}

#[cfg(test)] mod mac_adapter_migration_tests {
    use super::*;
    #[test] fn obsolete_lbp1_adapter_moves_to_rpcs3_without_losing_game_reference() {
        let mut c=Config::default();c.games[0].engine="lbprecomp".into();c.games[0].data="disc-folder".into();c.games[0].favorite=true;
        migrate_appearance(&mut c);
        assert_eq!(c.games[0].engine,"rpcs3");assert_eq!(c.games[0].data,"disc-folder");assert!(c.games[0].favorite);
    }
}
