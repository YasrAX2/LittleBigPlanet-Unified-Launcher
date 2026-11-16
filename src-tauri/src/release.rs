use crate::model::{err, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
pub fn repo(engine: &str) -> Result<&'static str> {
    match engine{"lbppc"=>Ok("emurphy04/LBPPC"),"rpcs3"=>Ok("RPCS3/rpcs3-binaries-win"),"lbprecomp"=>Ok("benjidotsh/LittleBigRecomp"),"replanet"=>Err("RePlanet has no verified public release source. Use RPCS3; check the author's announcement for news.".into()),_=>Err("Unknown adapter".into())}
}
pub fn client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent("PatchworkPod/1.0 (unofficial launcher)")
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(45))
        .redirect(reqwest::redirect::Policy::custom(|a| {
            let u = a.url();
            if a.previous().len() > 8 || !allowed_url(u) {
                a.stop()
            } else {
                a.follow()
            }
        }))
        .build()
        .map_err(err)
}
pub fn allowed_url(u: &reqwest::Url) -> bool {
    u.scheme() == "https"
        && matches!(
            u.host_str(),
            Some(
                "github.com"
                    | "api.github.com"
                    | "release-assets.githubusercontent.com"
                    | "objects.githubusercontent.com"
                    | "raw.githubusercontent.com"
            )
        )
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
    #[serde(default)]
    pub digest: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    #[serde(default)]
    pub body: Option<String>,
    pub html_url: String,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    pub assets: Vec<Asset>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub engine: String,
    pub version: String,
    pub page: String,
    pub notes: String,
    pub asset: Asset,
    pub checksum: Option<Asset>,
}
pub fn select_asset(r: &GithubRelease, engine: &str) -> Result<Asset> {
    let candidates: Vec<_> = r
        .assets
        .iter()
        .filter(|a| {
            let n = a.name.to_lowercase();
            (n.ends_with(".zip") || n.ends_with(".7z"))
                && (n.contains("windows") || n.contains("win64") || n.contains("win-x64"))
                && !n.contains("arm")
                && !n.contains("debug")
                && !n.contains("symbols")
        })
        .collect();
    if candidates.len() != 1 {
        return Err(format!("{} compatible Windows archives found. Open the official release page to review; nothing was installed.",candidates.len()));
    }
    let a = candidates[0].clone();
    let prefix = format!("https://github.com/{}/releases/download/", repo(engine)?);
    if !a.browser_download_url.starts_with(&prefix)
        || a.name.contains(['/', '\\', ':'])
        || a.size == 0
        || a.size > 4 * 1024 * 1024 * 1024
    {
        return Err("Release asset failed source or size checks.".into());
    }
    Ok(a)
}
pub fn latest(engine: &str) -> Result<Release> {
    let repo = repo(engine)?;
    let c = client()?;
    let response = c
        .get(format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ))
        .send()
        .map_err(err)?;
    if response.status() == 404 {
        return Err("No public release is currently published for this adapter. RPCS3 is available for LBP1 and LBP2.".into());
    }
    let r: GithubRelease = response
        .error_for_status()
        .map_err(err)?
        .json()
        .map_err(err)?;
    let asset = select_asset(&r, engine)?;
    let checksum = r
        .assets
        .iter()
        .find(|v| v.name == format!("{}.sha256", asset.name))
        .cloned();
    Ok(Release {
        engine: engine.into(),
        version: r.tag_name,
        page: r.html_url,
        notes: r.body.unwrap_or_default(),
        asset,
        checksum,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Requirements {
    pub title_ids: Vec<String>,
    pub version: String,
    pub checked: u64,
    pub source: String,
    pub live: bool,
}
impl Default for Requirements {
    fn default() -> Self {
        Self {
            title_ids: vec!["CUSA00473".into(), "CUSA00063".into()],
            version: "1.28".into(),
            checked: 0,
            source: "https://github.com/emurphy04/LBPPC/blob/main/README.md".into(),
            live: false,
        }
    }
}
pub fn requirements() -> Result<Requirements> {
    let text = client()?
        .get("https://raw.githubusercontent.com/emurphy04/LBPPC/main/README.md")
        .send()
        .map_err(err)?
        .error_for_status()
        .map_err(err)?
        .text()
        .map_err(err)?;
    parse_requirements(&text)
}
pub fn parse_requirements(text: &str) -> Result<Requirements> {
    let summary = text.split("## What you get").next().unwrap_or(text);
    let mut ids = vec![];
    for (at, _) in summary.match_indices("CUSA") {
        if let Some(v) = summary.get(at..at + 9) {
            if v[4..].bytes().all(|b| b.is_ascii_digit()) && !ids.contains(&v.to_string()) {
                ids.push(v.into())
            }
        }
    }
    let flat = summary.replace(['\n', '\r'], " ");
    let after = flat
        .split("version")
        .nth(1)
        .ok_or("Cannot parse upstream game requirements; cached requirements retained.")?;
    let version = after
        .trim_start()
        .split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .next()
        .unwrap_or("")
        .trim_end_matches('.');
    if ids.is_empty() || version.len() < 3 || !version.contains('.') {
        return Err("Upstream requirement format changed. Review the official guide.".into());
    }
    Ok(Requirements {
        title_ids: ids,
        version: version.into(),
        checked: crate::model::now(),
        live: true,
        ..Default::default()
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_requirements_live() {
        let r=parse_requirements("Supports CUSA00473 and CUSA00063, version\n1.28. You need your own files.\n## What you get").unwrap();
        assert_eq!(r.version, "1.28");
        assert_eq!(r.title_ids.len(), 2);
    }
    #[test]
    fn rejects_non_official_redirects() {
        for u in [
            "http://github.com/a",
            "https://github.com.evil.test/a",
            "https://example.com/a",
        ] {
            assert!(!allowed_url(&u.parse().unwrap()))
        }
    }
    #[test]
    fn windows_selection_only() {
        let mut r = GithubRelease {
            tag_name: "v1".into(),
            body: None,
            html_url: String::new(),
            prerelease: false,
            draft: false,
            assets: vec![Asset {
                name: "macos.zip".into(),
                size: 1,
                browser_download_url: String::new(),
                digest: None,
            }],
        };
        assert!(select_asset(&r, "lbppc").is_err());
        r.assets.push(Asset {
            name: "LBPPC-windows.zip".into(),
            size: 100,
            browser_download_url:
                "https://github.com/emurphy04/LBPPC/releases/download/v1/LBPPC-windows.zip".into(),
            digest: None,
        });
        assert_eq!(select_asset(&r, "lbppc").unwrap().size, 100);
    }
}
