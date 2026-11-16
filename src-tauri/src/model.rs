use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}, sync::Mutex};
pub type Result<T> = std::result::Result<T, String>;
pub fn err(e: impl std::fmt::Display) -> String { e.to_string() }
pub fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all="camelCase")]
pub struct Game {pub id:String,pub engine:String,pub data:String,pub update:String,pub title_id:String,pub region:String,pub version:String,pub favorite:bool,pub seconds:u64,pub last_played:u64,pub fullscreen:bool,pub artwork:String}
impl Default for Game { fn default()->Self {Self{id:"lbp1".into(),engine:"rpcs3".into(),data:String::new(),update:String::new(),title_id:String::new(),region:String::new(),version:String::new(),favorite:false,seconds:0,last_played:0,fullscreen:false,artwork:String::new()}} }
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default,rename_all="camelCase")]
pub struct Installation {pub engine:String,pub executable:String,pub version:String,pub managed:bool,pub root:String,pub rollback:String}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default,rename_all="camelCase")]
pub struct Config {pub schema:u32,pub theme:String,pub reduced_motion:bool,pub text_scale:u32,pub controller:bool,pub onboarding:bool,pub install_root:String,pub games:Vec<Game>,pub installations:BTreeMap<String,Installation>,pub activity:Vec<String>}
impl Default for Config {fn default()->Self {Self{schema:1,theme:"day".into(),reduced_motion:false,text_scale:100,controller:true,onboarding:false,install_root:String::new(),games:(1..=3).map(|n|Game{id:format!("lbp{n}"),engine:if n==3{"lbppc".into()}else{"rpcs3".into()},..Default::default()}).collect(),installations:BTreeMap::new(),activity:vec![]}}}
pub struct Store {pub dir:PathBuf,pub config:Mutex<Config>,pub busy:Mutex<bool>,pub running:Mutex<Vec<String>>,pub tasks:Mutex<BTreeMap<String,crate::download::Task>>}
impl Store {
 pub fn open(dir:PathBuf)->Result<Self>{fs::create_dir_all(&dir).map_err(err)?; let path=dir.join("config.json");let mut c:Config=if path.exists(){serde_json::from_slice(&fs::read(path).map_err(err)?).map_err(|_|"Configuration is unreadable. Restore config.backup.json before starting.".to_string())?}else{Config::default()};validate_config(&c)?;if c.install_root.is_empty(){c.install_root=dir.join("engines").to_string_lossy().into()};Ok(Self{dir,config:Mutex::new(c),busy:Mutex::new(false),running:Mutex::new(vec![]),tasks:Mutex::new(BTreeMap::new())})}
 pub fn change(&self, f:impl FnOnce(&mut Config)->Result<()>)->Result<Config>{let mut guard=self.config.lock().map_err(err)?;let mut c=guard.clone();f(&mut c)?;validate_config(&c)?;atomic_json(&self.dir.join("config.json"),&c,true)?;*guard=c.clone();Ok(c)}
 pub fn get(&self)->Config{self.config.lock().unwrap().clone()}
}
pub fn validate_config(c:&Config)->Result<()>{if c.schema!=1||c.games.len()!=3||!["day","night","system"].contains(&c.theme.as_str())||!(85..=150).contains(&c.text_scale){return Err("Unsupported configuration.".into())}for id in ["lbp1","lbp2","lbp3"]{if c.games.iter().filter(|g|g.id==id).count()!=1{return Err("Each game must appear exactly once.".into())}}for g in &c.games {if !["lbppc","rpcs3","lbprecomp","replanet"].contains(&g.engine.as_str()) {return Err("Unknown game engine.".into())}}Ok(())}
pub fn atomic_json(path:&Path,value:&impl Serialize,backup:bool)->Result<()> {let parent=path.parent().ok_or("Invalid configuration path")?;fs::create_dir_all(parent).map_err(err)?;let mut tmp=tempfile::NamedTempFile::new_in(parent).map_err(err)?;serde_json::to_writer_pretty(&mut tmp,value).map_err(err)?;tmp.as_file().sync_all().map_err(err)?;if backup&&path.is_file(){fs::copy(path,path.with_extension("backup.json")).map_err(err)?;}tmp.persist(path).map_err(err)?;Ok(())}
pub fn activity(c:&mut Config, text:&str){c.activity.insert(0,text.into());c.activity.truncate(24)}
pub fn game_mut<'a>(c:&'a mut Config,id:&str)->Result<&'a mut Game>{c.games.iter_mut().find(|g|g.id==id).ok_or("Unknown game".into())}
