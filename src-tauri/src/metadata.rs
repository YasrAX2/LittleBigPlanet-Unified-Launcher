use crate::model::{err, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection {
    pub path: String,
    pub title_id: String,
    pub title: String,
    pub region: String,
    pub version: String,
    pub valid: bool,
    pub messages: Vec<String>,
}
fn u32_at(data: &[u8], at: usize) -> Result<usize> {
    Ok(u32::from_le_bytes(
        data.get(at..at + 4)
            .ok_or("Truncated SFO")?
            .try_into()
            .map_err(err)?,
    ) as usize)
}
pub fn parse_sfo(d: &[u8]) -> Result<BTreeMap<String, String>> {
    if d.len() < 20 || d.get(..4) != Some(b"\0PSF") {
        return Err("Not a PlayStation PARAM.SFO file.".into());
    }
    let keys = u32_at(d, 8)?;
    let values = u32_at(d, 12)?;
    let count = u32_at(d, 16)?;
    if count > 4096 || 20 + count * 16 > d.len() {
        return Err("Invalid SFO index.".into());
    }
    let mut out = BTreeMap::new();
    for i in 0..count {
        let at = 20 + i * 16;
        let ko = u16::from_le_bytes(d[at..at + 2].try_into().unwrap()) as usize;
        let len = u32_at(d, at + 4)?;
        let vo = u32_at(d, at + 12)?;
        let key_start = keys.checked_add(ko).ok_or("SFO overflow")?;
        let key_tail = d.get(key_start..).ok_or("Invalid SFO key")?;
        let key_end = key_tail
            .iter()
            .position(|c| *c == 0)
            .ok_or("Unterminated SFO key")?;
        let key = std::str::from_utf8(&key_tail[..key_end]).map_err(err)?;
        let start = values.checked_add(vo).ok_or("SFO overflow")?;
        let end = start.checked_add(len).ok_or("SFO overflow")?;
        let value = d.get(start..end).ok_or("Invalid SFO value")?;
        if d[at + 2..at + 4] == [4, 2] {
            out.insert(
                key.into(),
                String::from_utf8_lossy(value)
                    .trim_end_matches('\0')
                    .to_string(),
            );
        }
    }
    Ok(out)
}
pub fn metadata_path(p: &Path) -> Option<PathBuf> {
    [
        p.join("PS3_GAME/PARAM.SFO"),
        p.join("PARAM.SFO"),
        p.join("sce_sys/param.sfo"),
        p.join("sce_sys/PARAM.SFO"),
        p.join("Sc0/param.sfo"),
    ]
    .into_iter()
    .find(|v| v.is_file())
}
pub fn sfo_at(p: &Path) -> Result<BTreeMap<String, String>> {
    let s = metadata_path(p).ok_or(
        "No PARAM.SFO found. Choose the extracted game folder, not an ISO, PKG or archive.",
    )?;
    if fs::metadata(&s).map_err(err)?.len() > 1024 * 1024 {
        return Err("PARAM.SFO is too large.".into());
    }
    parse_sfo(&fs::read(s).map_err(err)?)
}
pub fn boot_path(p: &Path) -> Option<PathBuf> {
    [
        p.join("PS3_GAME/USRDIR/EBOOT.BIN"),
        p.join("USRDIR/EBOOT.BIN"),
    ]
    .into_iter()
    .find(|v| v.is_file())
}
pub fn inspect(
    id: &str,
    p: &Path,
    is_update: bool,
    requirements: &crate::release::Requirements,
) -> Result<Inspection> {
    if !p.is_dir() {
        return Err("This folder is missing. Reconnect its drive or choose another folder.".into());
    }
    let m = sfo_at(p)?;
    let title_id = m.get("TITLE_ID").cloned().unwrap_or_default();
    let title = m.get("TITLE").cloned().unwrap_or_default();
    let version = m
        .get("APP_VER")
        .or(m.get("VERSION"))
        .cloned()
        .unwrap_or_default();
    let mut r = Inspection {
        path: p.canonicalize().map_err(err)?.to_string_lossy().into(),
        title_id: title_id.clone(),
        title,
        version,
        valid: true,
        ..Default::default()
    };
    r.region = if title_id == "CUSA00473"
        || title_id.starts_with("BCUS")
        || title_id.starts_with("BLUS")
        || title_id.starts_with("NPUB")
    {
        "US"
    } else if title_id == "CUSA00063"
        || title_id.starts_with("BCES")
        || title_id.starts_with("BLES")
        || title_id.starts_with("NPEB")
    {
        "Europe"
    } else if title_id.starts_with("BCJS")
        || title_id.starts_with("BLJS")
        || title_id.starts_with("NPJB")
    {
        "Japan"
    } else {
        "Other / unknown"
    }
    .into();
    let title_norm = r.title.to_lowercase().replace([' ', '™', '®', ':'], "");
    if id == "lbp3" {
        if !requirements.title_ids.contains(&title_id) {
            r.valid = false;
            r.messages.push(format!(
                "Unsupported region. Upstream lists: {}.",
                requirements.title_ids.join(", ")
            ))
        }
        if is_update && r.version.trim_start_matches('0') != requirements.version {
            r.valid = false;
            r.messages.push(format!(
                "Expected update {}, found {}.",
                requirements.version, r.version
            ))
        }
        if !p.join("eboot.bin").is_file() {
            r.valid = false;
            r.messages
                .push("Missing eboot.bin. Select the extracted PS4 base-game folder.".into())
        }
        if !is_update && !p.join("base_001.farc").is_file() {
            r.valid = false;
            r.messages.push(
                "Missing base_001.farc. Choose the full base game, not an update folder.".into(),
            )
        }
        if r.version.trim_start_matches('0') != requirements.version && !is_update {
            r.messages.push(format!(
                "Add a separate {} update folder before playing.",
                requirements.version
            ))
        }
    } else {
        let number = if id == "lbp2" { "2" } else { "1" };
        let matches = if number == "2" {
            title_norm.contains("littlebigplanet2")
        } else {
            title_norm.contains("littlebigplanet")
                && !title_norm.contains("littlebigplanet2")
                && !title_norm.contains("littlebigplanet3")
                && !title_norm.contains("karting")
        };
        if !matches || title_id.starts_with("CUSA") {
            r.valid = false;
            r.messages.push("This metadata belongs to a different game. Choose the matching LittleBigPlanet folder.".into())
        }
        if boot_path(p).is_none() {
            r.valid = false;
            r.messages.push("Missing USRDIR/EBOOT.BIN. Install digital packages through RPCS3 first, then select the installed game folder.".into())
        }
        r.messages.push("File checks cannot prove a dump is decrypted or playable. RPCS3 needs official PS3 firmware and any required game license.".into());
    }
    if r.version.is_empty() {
        r.messages
            .push("Game version was not provided in PARAM.SFO.".into())
    }
    Ok(r)
}
pub fn windows_exe(p: &Path) -> Result<()> {
    if !p.is_file()
        || p.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase()
            != "exe"
    {
        return Err("Choose a Windows .exe file.".into());
    }
    let mut f = fs::File::open(p).map_err(err)?;
    let mut h = [0; 64];
    f.read_exact(&mut h)
        .map_err(|_| "Executable is truncated.")?;
    if &h[..2] != b"MZ" {
        return Err("Not a Windows executable. macOS and Linux builds cannot run here.".into());
    }
    let offset = u32::from_le_bytes(h[60..64].try_into().unwrap());
    f.seek(SeekFrom::Start(offset as u64)).map_err(err)?;
    let mut pe = [0; 6];
    f.read_exact(&mut pe).map_err(err)?;
    if &pe[..4] != b"PE\0\0" || pe[4..6] != [0x64, 0x86] {
        return Err("A Windows x64 PE executable is required.".into());
    }
    Ok(())
}
#[cfg(test)]
pub fn fixture(title: &str, id: &str, version: &str) -> Vec<u8> {
    let vals = [("TITLE", title), ("TITLE_ID", id), ("APP_VER", version)];
    let mut keys = vec![];
    let mut values = vec![];
    let mut entries = vec![];
    for (k, v) in vals {
        entries.extend_from_slice(&(keys.len() as u16).to_le_bytes());
        entries.extend([4, 2]);
        entries.extend_from_slice(&((v.len() + 1) as u32).to_le_bytes());
        entries.extend_from_slice(&((v.len() + 1) as u32).to_le_bytes());
        entries.extend_from_slice(&(values.len() as u32).to_le_bytes());
        keys.extend(k.as_bytes());
        keys.push(0);
        values.extend(v.as_bytes());
        values.push(0);
    }
    let mut d = b"\0PSF".to_vec();
    d.extend(0x101u32.to_le_bytes());
    d.extend(68u32.to_le_bytes());
    d.extend((68u32 + keys.len() as u32).to_le_bytes());
    d.extend(3u32.to_le_bytes());
    d.extend(entries);
    d.extend(keys);
    d.extend(values);
    d
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_and_rejects_truncated() {
        let b = fixture("LittleBigPlanet 2", "BCES00850", "01.33");
        assert_eq!(parse_sfo(&b).unwrap()["TITLE_ID"], "BCES00850");
        for n in 0..b.len() {
            assert!(parse_sfo(&b[..n]).is_err());
        }
    }
    #[test]
    fn missing_game_rejected() {
        let t = tempfile::tempdir().unwrap();
        assert!(inspect("lbp1", t.path(), false, &Default::default()).is_err());
    }
    #[test]
    fn mismatched_title_invalid() {
        let t = tempfile::tempdir().unwrap();
        fs::write(
            t.path().join("PARAM.SFO"),
            fixture("LittleBigPlanet 2", "BCES00850", "01.33"),
        )
        .unwrap();
        assert!(
            !inspect("lbp1", t.path(), false, &Default::default())
                .unwrap()
                .valid
        );
    }
    #[test]
    fn non_windows_rejected() {
        let t = tempfile::tempdir().unwrap();
        fs::write(t.path().join("mac.exe"), [0u8; 64]).unwrap();
        assert!(windows_exe(&t.path().join("mac.exe")).is_err());
    }
}
