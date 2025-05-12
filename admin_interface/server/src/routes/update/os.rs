use super::util::{parse_version, SemVer};
use crate::problem::Problem;
use crate::{problem, to_problem};
use serde::Deserialize;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::{fs, io};

pub fn get_current_os_version() -> Result<SemVer, io::Error> {
    if cfg!(not(target_os = "linux")) {
        return Ok((0, 0, 0, None));
    }
    let version = fs::read_to_string("/etc/os-release")?
        .lines()
        .filter(|line| line.starts_with("VERSION_ID="))
        .map(|line| line.trim_start_matches("VERSION_ID=").trim_matches('"'))
        .nth(0)
        .ok_or(io::ErrorKind::NotFound)?
        .to_owned();

    parse_version(&version).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Invalid version",
    ))
}

#[allow(dead_code)]
pub fn get_current_app_version() -> Result<(u32, u32, u32, Option<String>), io::Error> {
    Ok((0, 0, 0, None))
}

pub fn get_root_partitions() -> Result<(BlockDevice, BlockDevice), Problem> {
    if cfg!(not(target_os = "linux")) {
        return Err(problem!("Update not supported on non-linux systems", 400));
    }
    let target_partition_key = {
        let partition_file = Path::new("/boot/rootfs_partition");
        if !partition_file.exists() {
            "a".to_string()
        } else {
            fs::read_to_string(partition_file).unwrap_or("a".to_string())
        }
    };

    let partitions = get_partitions().map_err(to_problem!("Failed to get partitions", 500))?;

    let rootfs_a = partitions
        .iter()
        .find(|partition| partition.part_label == "rootfs_a")
        .map(|partition| partition.to_owned());

    let rootfs_b = partitions
        .iter()
        .find(|partition| partition.part_label == "rootfs_b")
        .map(|partition| partition.to_owned());

    if rootfs_a.is_none() || rootfs_b.is_none() {
        return Err(problem!(
            "Could not identify update partitions",
            500,
            "Available partitions: {:?}",
            partitions
        ));
    }

    if target_partition_key == "a" {
        Ok((rootfs_a.unwrap(), rootfs_b.unwrap()))
    } else {
        Ok((rootfs_b.unwrap(), rootfs_a.unwrap()))
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct BlockDevice {
    #[serde(rename = "partn")]
    pub part_number: i32,
    #[serde(rename = "partlabel")]
    pub part_label: String,

    pub path: String,
    #[serde(rename = "kname")]
    pub device_name: String,
    #[serde(rename = "mountpoint")]
    pub mount_point: Option<String>,
}

#[derive(Deserialize)]
pub struct LsblkResponse {
    #[serde(rename = "blockdevices")]
    pub devices: Vec<BlockDevice>,
}

pub fn get_partitions() -> Result<Vec<BlockDevice>, Problem> {
    if cfg!(not(target_os = "linux")) {
        return Ok(vec![]);
    }

    let mut lsblk_output = String::new();
    Command::new("lsblk")
        .arg("-J")
        .arg("-o")
        .arg("PARTN,PARTLABEL,PATH,KNAME,MOUNTPOINT")
        .spawn()
        .map_err(to_problem!("Failed to lookup partitions", 500))?
        .stdout
        .unwrap()
        .read_to_string(&mut lsblk_output)
        .map_err(to_problem!("Failed to read stdout of lsblk command", 500))?;

    let lsblk_response: LsblkResponse = serde_json::from_str(&lsblk_output)
        .map_err(to_problem!("Failed to parse lsblk output", 500))?;

    Ok(lsblk_response.devices)
}
