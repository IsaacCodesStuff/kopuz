pub mod activity;
pub mod album;
pub mod artist;

/// Convert a `TrackId::uid_path()` (the still-`PathBuf`-keyed selection form) back
/// to a playlist ref — the item id for a server track, the path string for local.
/// Source-uniform: `from_legacy_path` parses the `"service:id"` uid or a real path,
/// so callers never branch local-vs-server (replaces the old `parse_item_id` split).
pub(crate) fn ref_from_uid_path(path: &std::path::Path) -> String {
    reader::models::TrackId::from_legacy_path(&path.to_string_lossy())
        .key()
        .into_owned()
}

pub mod favorites;
pub mod favorites_body;
pub mod home;
pub mod home_body;
pub mod layout;
pub mod library;
pub mod playlists;
pub mod radio;
pub mod search;
pub mod server;
pub mod settings;
#[cfg(not(target_os = "android"))]
pub mod theme_editor;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub mod ytdlp;
