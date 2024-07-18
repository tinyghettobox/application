#[path = "src/util.rs"]
mod util;
#[path = "src/model/mod.rs"]
mod model;

use model::library_entry::{Model as LibraryEntry, Variant};
use model::spotify_config::Model as SpotifyConfig;
use model::system_config::Model as SystemConfig;
use model::track_source::Model as TrackSource;
use regex::Regex;
use ts_rs::TS;

fn main() {
    println!("cargo:rerun-if-changed=src/model");

    let regex = Regex::new("(?<a>[a-z])_(?<b>[a-z])").unwrap();
    let replacer = |caps: &regex::Captures| format!("{}{}", &caps["a"], &caps["b"].to_uppercase());

    let variant = Variant::export_to_string().unwrap();
    std::fs::write("types/Variant.d.ts", regex.replace_all(&variant, replacer).to_string())
        .expect("Failed to write file");

    let system_config = SystemConfig::export_to_string().unwrap();
    std::fs::write(
        "types/SystemConfig.d.ts",
        regex.replace_all(&system_config, replacer).to_string(),
    )
    .expect("Failed to write file");

    let spotify_config = SpotifyConfig::export_to_string().unwrap();
    std::fs::write(
        "types/SpotifyConfig.d.ts",
        regex.replace_all(&spotify_config, replacer).to_string(),
    )
    .expect("Failed to write file");

    let track_source = TrackSource::export_to_string().unwrap();
    std::fs::write(
        "types/TrackSource.d.ts",
        regex.replace_all(&track_source, replacer).to_string(),
    )
    .expect("Failed to write file");

    // Sea orm requires id field to be ignored for serde deserialize which leads to id field missing in types.
    // We are adding the id field here manually.
    let library_entry = LibraryEntry::export_to_string().unwrap();
    let library_entry = library_entry.replace("parent_id?:", "id?: number; parent_id?:");
    std::fs::write(
        "types/LibraryEntry.d.ts",
        regex.replace_all(&library_entry, replacer).to_string(),
    )
    .expect("Failed to write file");
}
