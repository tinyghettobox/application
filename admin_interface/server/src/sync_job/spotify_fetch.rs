use rspotify::{
    model::{AlbumId, ArtistId, PlaylistId, ShowId},
    prelude::{BaseClient, Id},
    AuthCodeSpotify,
};
use tracing::debug;

use super::SyncError;

/// Normalised representation of a single Spotify item returned by any child endpoint.
#[derive(Debug, Clone)]
pub struct SpotifyItem {
    pub spotify_id: String,
    pub name: String,
    pub image_url: Option<String>,
    /// Spotify type of *this* item (e.g. `"album"`, `"track"`, `"episode"`).
    pub spotify_type: String,
    pub disc_number: Option<u32>,
    pub track_number: Option<u32>,
    pub release_date: Option<String>,
}

/// Returns true for Spotify types that have children and should be stored as
/// `Variant::Folder` library entries.
pub fn is_container(spotify_type: &str) -> bool {
    matches!(spotify_type, "artist" | "album" | "playlist" | "show")
}

/// Fetches every page of children for a Spotify container and returns them as a
/// flat, ordered list. The list order matches the Spotify API order (newest-first
/// for artists, track-number order for albums, etc.).
pub fn fetch_all_children(
    spotify_id: &str,
    spotify_type: &str,
    spotify: &AuthCodeSpotify,
) -> Result<Vec<SpotifyItem>, SyncError> {
    const PAGE_SIZE: u32 = 50;
    let mut items: Vec<SpotifyItem> = Vec::new();
    let mut offset: u32 = 0;

    loop {
        debug!(spotify_id, spotify_type, offset, "fetching page from Spotify");
        let (page_items, total) = fetch_page(spotify_id, spotify_type, spotify, offset, PAGE_SIZE)?;
        let fetched = page_items.len() as u32;
        debug!(spotify_id, fetched, total, "got page of {} items (total={})", fetched, total);
        items.extend(page_items);
        offset += fetched;

        if fetched == 0 || offset >= total {
            break;
        }
    }

    Ok(items)
}

fn fetch_page(
    spotify_id: &str,
    spotify_type: &str,
    spotify: &AuthCodeSpotify,
    offset: u32,
    limit: u32,
) -> Result<(Vec<SpotifyItem>, u32), SyncError> {
    match spotify_type {
        "artist" => fetch_artist_albums(spotify_id, spotify, offset, limit),
        "album" => fetch_album_tracks(spotify_id, spotify, offset, limit),
        "playlist" => fetch_playlist_items(spotify_id, spotify, offset, limit),
        "show" => fetch_show_episodes(spotify_id, spotify, offset, limit),
        other => Err(SyncError::Spotify(format!("Unknown spotify_type: {}", other))),
    }
}

fn fetch_artist_albums(
    spotify_id: &str,
    spotify: &AuthCodeSpotify,
    offset: u32,
    limit: u32,
) -> Result<(Vec<SpotifyItem>, u32), SyncError> {
    let id = ArtistId::from_id(spotify_id).map_err(|e| SyncError::Spotify(e.to_string()))?;
    let page = spotify
        .artist_albums_manual(id, vec![], None, Some(limit), Some(offset))
        .map_err(map_rspotify_error)?;
    let total = page.total;
    let items = page
        .items
        .into_iter()
        .filter_map(|album| {
            Some(SpotifyItem {
                spotify_id: album.id?.id().to_string(),
                name: album.name,
                image_url: album.images.first().map(|i| i.url.clone()),
                spotify_type: "album".to_string(),
                disc_number: None,
                track_number: None,
                release_date: album.release_date,
            })
        })
        .collect();
    Ok((items, total))
}

fn fetch_album_tracks(
    spotify_id: &str,
    spotify: &AuthCodeSpotify,
    offset: u32,
    limit: u32,
) -> Result<(Vec<SpotifyItem>, u32), SyncError> {
    let id = AlbumId::from_id(spotify_id).map_err(|e| SyncError::Spotify(e.to_string()))?;
    let page = spotify
        .album_track_manual(id, None, Some(limit), Some(offset))
        .map_err(map_rspotify_error)?;
    let total = page.total;
    let items = page
        .items
        .into_iter()
        .filter_map(|track| {
            Some(SpotifyItem {
                spotify_id: track.id?.id().to_string(),
                name: track.name,
                image_url: None, // album art lives on the parent album entry
                spotify_type: "track".to_string(),
                disc_number: Some(track.disc_number as u32),
                track_number: Some(track.track_number),
                release_date: None,
            })
        })
        .collect();
    Ok((items, total))
}

fn fetch_playlist_items(
    spotify_id: &str,
    spotify: &AuthCodeSpotify,
    offset: u32,
    limit: u32,
) -> Result<(Vec<SpotifyItem>, u32), SyncError> {
    let id = PlaylistId::from_id(spotify_id).map_err(|e| SyncError::Spotify(e.to_string()))?;
    let page = spotify
        .playlist_items_manual(id, None, None, Some(limit), Some(offset))
        .map_err(map_rspotify_error)?;
    let total = page.total;
    let items = page
        .items
        .into_iter()
        .filter_map(|pi| {
            let playable = pi.track?;
            match playable {
                rspotify::model::PlayableItem::Track(t) => Some(SpotifyItem {
                    spotify_id: t.id?.id().to_string(),
                    name: t.name,
                    image_url: t.album.images.first().map(|i| i.url.clone()),
                    spotify_type: "track".to_string(),
                    disc_number: Some(t.disc_number as u32),
                    track_number: Some(t.track_number),
                    release_date: None,
                }),
                rspotify::model::PlayableItem::Episode(e) => Some(SpotifyItem {
                    spotify_id: e.id.id().to_string(),
                    name: e.name,
                    image_url: e.images.first().map(|i| i.url.clone()),
                    spotify_type: "episode".to_string(),
                    disc_number: None,
                    track_number: None,
                    release_date: Some(e.release_date),
                }),
            }
        })
        .collect();
    Ok((items, total))
}

fn fetch_show_episodes(
    spotify_id: &str,
    spotify: &AuthCodeSpotify,
    offset: u32,
    limit: u32,
) -> Result<(Vec<SpotifyItem>, u32), SyncError> {
    let id = ShowId::from_id(spotify_id).map_err(|e| SyncError::Spotify(e.to_string()))?;
    let page = spotify
        .get_shows_episodes_manual(id, None, Some(limit), Some(offset))
        .map_err(map_rspotify_error)?;
    let total = page.total;
    let items = page
        .items
        .into_iter()
        .map(|ep| SpotifyItem {
            spotify_id: ep.id.id().to_string(),
            name: ep.name,
            image_url: ep.images.first().map(|i| i.url.clone()),
            spotify_type: "episode".to_string(),
            disc_number: None,
            track_number: None,
            release_date: Some(ep.release_date),
        })
        .collect();
    Ok((items, total))
}

pub fn map_rspotify_error(e: rspotify::ClientError) -> SyncError {
    let msg = e.to_string();
    if msg.contains("429") || msg.to_lowercase().contains("too many requests") {
        let secs = parse_retry_after(&msg).unwrap_or(60);
        return SyncError::RateLimited(secs);
    }
    SyncError::Spotify(msg)
}

fn parse_retry_after(msg: &str) -> Option<u64> {
    let re = regex_lite::Regex::new(r"(?i)retry.after[:\s]+(\d+)").ok()?;
    re.captures(msg)?.get(1)?.as_str().parse().ok()
}
