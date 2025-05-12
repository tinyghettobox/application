use serde::{Deserialize, Serialize};
use std::io;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReleaseAsset {
    pub id: u64,
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubRelease {
    pub id: u64,
    pub tag_name: String,
    pub prerelease: bool,
    pub name: String,
    pub body: String,
    pub assets: Vec<ReleaseAsset>,
}

pub fn fetch_available_releases(repo_url: &str) -> Result<Vec<GithubRelease>, io::Error> {
    ureq::get(format!("{}/releases", repo_url).as_str())
        .call()
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to fetch releases: {}", error),
            )
        })?
        .into_json::<Vec<GithubRelease>>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid JSON response"))
}
