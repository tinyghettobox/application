use regex::Regex;
use ts_rs::TS;

use model::library_entry::{Model as LibraryEntry, Variant};
use model::spotify_config::Model as SpotifyConfig;
use model::system_config::Model as SystemConfig;
use model::track_source::Model as TrackSource;

#[path = "src/util.rs"]
mod util;
#[path = "src/model/mod.rs"]
mod model;

fn replace_snake_to_upper(content: String) -> String {
    let regex = Regex::new("(?<a>[a-z])_(?<b>[a-z])").unwrap();
    regex.replace_all(&content, |caps: &regex::Captures| format!("{}{}", &caps["a"], &caps["b"].to_uppercase())).to_string()
}

fn replace_id_to_optional(content: String) -> String {
    content.replace("id: number", "id?: number")
}

fn fix_types(content: String) -> String {
    replace_id_to_optional(replace_snake_to_upper(content))
}

fn main() {
    println!("cargo:rerun-if-changed=src/model");

    let variant = Variant::export_to_string().unwrap();
    std::fs::write("types/Variant.d.ts", fix_types(variant))
        .expect("Failed to write file");

    let system_config = SystemConfig::export_to_string().unwrap();
    std::fs::write("types/SystemConfig.d.ts", fix_types(system_config))
        .expect("Failed to write file");

    let spotify_config = SpotifyConfig::export_to_string().unwrap();
    std::fs::write("types/SpotifyConfig.d.ts", fix_types(spotify_config))
        .expect("Failed to write file");

    let track_source = TrackSource::export_to_string().unwrap();
    std::fs::write("types/TrackSource.d.ts", fix_types(track_source))
        .expect("Failed to write file");

    // Sea orm requires id field to be ignored for serde deserialize which leads to id field missing in types.
    // We are adding the id field here manually.
    let library_entry = LibraryEntry::export_to_string().unwrap();
    std::fs::write("types/LibraryEntry.d.ts", fix_types(library_entry))
        .expect("Failed to write file");
}
