use regex::Regex;
use ts_rs::TS;

use model::library_entry;
use model::spotify_config;
use model::system_config;
use model::track_source;

#[path = "src/model/mod.rs"]
mod model;
#[path = "src/util.rs"]
mod util;

fn replace_snake_to_upper(content: String) -> String {
    let regex = Regex::new("(?<a>[a-z])_(?<b>[a-z])").unwrap();
    regex
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}{}", &caps["a"], &caps["b"].to_uppercase())
        })
        .to_string()
}

fn fix_types(content: String) -> String {
    replace_snake_to_upper(content)
}

fn main() {
    println!("cargo:rerun-if-changed=src/model");

    let variant = library_entry::Variant::export_to_string().unwrap();
    std::fs::write("types/Variant.d.ts", fix_types(variant)).expect("Failed to write file");

    let system_config = system_config::Model::export_to_string().unwrap();
    std::fs::write("types/SystemConfig.d.ts", fix_types(system_config))
        .expect("Failed to write file");

    let spotify_config = spotify_config::Model::export_to_string().unwrap();
    std::fs::write("types/SpotifyConfig.d.ts", fix_types(spotify_config))
        .expect("Failed to write file");

    let track_source = track_source::Model::export_to_string().unwrap();
    let new_track_source = track_source::CreateModel::export_to_string().unwrap();
    std::fs::write(
        "types/TrackSource.d.ts",
        fix_types(format!("{}\n\n{}", track_source, new_track_source)),
    )
    .expect("Failed to write file");

    // Sea orm requires id field to be ignored for serde deserialize which leads to id field missing in types.
    // We are adding the id field here manually.
    let library_entry = library_entry::Model::export_to_string().unwrap();
    let new_library_entry = library_entry::CreateModel::export_to_string().unwrap();
    std::fs::write(
        "types/LibraryEntry.d.ts",
        fix_types(format!("{}\n\n{}", library_entry, new_library_entry)),
    )
    .expect("Failed to write file");
}
