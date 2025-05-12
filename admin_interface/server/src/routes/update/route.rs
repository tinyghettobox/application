use super::github::{fetch_available_releases, GithubRelease};
use super::os::{get_current_os_version, get_root_partitions};
use super::util::{compare_version, parse_version};
use crate::problem::Problem;
use crate::{problem, to_problem};
use actix_web::web::Json;
use actix_web::{get, post, HttpResponse, Responder};
use serde::Deserialize;
use std::cmp::Ordering;

const OS_REPO_URL: &str = "https://api.github.com/repos/tinyghettobox/operating-system";

#[get("/api/update/operating-system")]
pub async fn get_operating_system_update_version() -> actix_web::Result<impl Responder> {
    let current_version =
        get_current_os_version().map_err(to_problem!("Could not get current os version", 500))?;

    let available_versions = fetch_available_releases(OS_REPO_URL)
        .map_err(to_problem!("Could not fetch available releases", 500))?;

    // Filter versions that are newer than the current version
    let newer_versions = available_versions
        .into_iter()
        .filter(|release| {
            if let Some(release_version) = parse_version(&release.tag_name) {
                if compare_version(&release_version, &current_version) == Ordering::Greater {
                    return true;
                }
            }
            false
        })
        .collect::<Vec<GithubRelease>>();

    Ok(HttpResponse::Ok().json(newer_versions))
}

#[derive(Deserialize)]
struct UpdateOperatingSystem {
    version: String,
}
#[post("/update/operating-system")]
pub async fn update_operating_system(
    payload: Json<UpdateOperatingSystem>,
) -> Result<HttpResponse, Problem> {
    let release = fetch_available_releases(OS_REPO_URL)
        .map_err(to_problem!("Could not fetch available releases", 500))?
        .into_iter()
        .find(|release| release.tag_name == payload.version)
        .ok_or(problem!("No release for this version found", 404))?;

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with("update.tar.gz"))
        .ok_or(problem!("The release has no update archive", 400))?;

    let (_used_partition, _target_partition) =
        get_root_partitions().map_err(to_problem!("Could not get target partition", 500))?;

    // Download the update archive to /srv/update.tar.gz
    // Download the update archive using ureq
    let response = ureq::get(&asset.browser_download_url)
        .call()
        .map_err(to_problem!("Failed to download the update archive", 500))?;

    let archive_path = "/srv/update.tar.gz";
    let mut file = std::fs::File::create(archive_path)
        .map_err(to_problem!("Failed to create archive file", 500))?;

    std::io::copy(&mut response.into_reader(), &mut file)
        .map_err(to_problem!("Failed to write archive to file", 500))?;

    // Extract the archive using Linux commands
    let extract_path = "/srv/update";
    std::fs::create_dir_all(extract_path)
        .map_err(|_| problem!("Failed to create extraction directory", 500))?;

    let output = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(archive_path)
        .arg("-C")
        .arg(extract_path)
        .output()
        .map_err(|_| problem!("Failed to execute tar command", 500))?;

    if !output.status.success() {
        return Err(problem!(
            "Failed to extract archive",
            500,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(HttpResponse::Ok().body("asd"))
}
