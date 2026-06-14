//! SoundCloud integration via the public `api-v2` web-player endpoint.
//!
//! Unlike Apple Music / Spotify, SoundCloud streams are **not** DRM'd: every
//! publicly streamable track exposes a `progressive` transcoding that resolves
//! to a plain MP3 URL on `cf-media.sndcdn.com`. Those flow through the exact
//! same Symphonia/cpal decode path as every other backend, identically on
//! Linux/macOS/Windows. So this is genuine full-track playback, not previews.
//!
//! There is no public, registerable API key anymore, so — like every
//! SoundCloud client (`yt-dlp`, `scdl`, …) — we lift the web player's
//! `client_id` out of its JS bundles at runtime and cache it. The id rotates
//! occasionally; any `4xx` triggers one forced re-scrape before giving up.
//!
//! Track encoding mirrors the YouTube Music backend: the path is
//! `soundcloud:<trackId>:urlhex_<artwork-url-hex>` so the shared cover resolver
//! (`utils::jellyfin_image`) can decode artwork synchronously, and the stream
//! URL is resolved lazily via [`resolve_stream`] (the controller tags it with a
//! `__SC_PENDING:` sentinel, just like `__YT_PENDING:`).

use std::collections::HashSet;
use std::path::PathBuf;

use reader::models::Track;
use serde_json::Value;
use tokio::sync::Mutex;

pub const SOURCE_PREFIX: &str = "soundcloud";

/// SoundCloud's internal web-player API. Keyless apart from the scraped
/// `client_id` query param.
const API_V2: &str = "https://api-v2.soundcloud.com";

/// The web player whose HTML/JS bundles carry the `client_id`.
const WEB_HOST: &str = "https://soundcloud.com";

/// Process-wide cached `client_id`. `None` until first scrape; a `4xx` from the
/// API forces a re-scrape (the id rotates server-side every so often).
static CLIENT_ID: Mutex<Option<String>> = Mutex::const_new(None);

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap_or_default()
}

/// Resolve the web player's `client_id`, scraping it on first use (or when
/// `force` is set after a stale-id error) and caching it for the process.
async fn client_id(http: &reqwest::Client, force: bool) -> Result<String, String> {
    let mut guard = CLIENT_ID.lock().await;
    if !force
        && let Some(id) = guard.as_ref()
    {
        return Ok(id.clone());
    }
    let id = scrape_client_id(http).await?;
    *guard = Some(id.clone());
    Ok(id)
}

/// Pull a fresh `client_id` out of the web player's JavaScript bundles.
///
/// The homepage references a handful of `*.sndcdn.com/assets/*.js` chunks; the
/// id lives in one of them as `client_id:"<32+ chars>"`. It's usually in one of
/// the later bundles, so we scan them newest-first and stop at the first hit.
async fn scrape_client_id(http: &reqwest::Client) -> Result<String, String> {
    let html = http
        .get(WEB_HOST)
        .send()
        .await
        .map_err(|e| format!("SoundCloud homepage HTTP: {e}"))?
        .error_for_status()
        .map_err(|e| format!("SoundCloud homepage HTTP: {e}"))?
        .text()
        .await
        .map_err(|e| format!("SoundCloud homepage body: {e}"))?;

    let mut scripts = Vec::new();
    for chunk in html.split("<script") {
        if let Some(src) = extract_attr(chunk, "src")
            && src.contains("sndcdn.com/assets/")
            && src.ends_with(".js")
        {
            scripts.push(src.to_string());
        }
    }

    for src in scripts.iter().rev() {
        if let Ok(resp) = http.get(src).send().await
            && let Ok(js) = resp.text().await
            && let Some(id) = find_client_id(&js)
        {
            return Ok(id);
        }
    }
    Err("SoundCloud: couldn't extract a client_id from the web player".to_string())
}

/// Read the value of an HTML `attr="…"` from a fragment, if present.
fn extract_attr<'a>(chunk: &'a str, attr: &str) -> Option<&'a str> {
    let key = format!("{attr}=\"");
    let start = chunk.find(&key)? + key.len();
    let rest = &chunk[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// Locate a `client_id` literal in a JS bundle. SoundCloud emits it as
/// `client_id:"…"` (and occasionally as a quoted JSON key); both are covered.
fn find_client_id(js: &str) -> Option<String> {
    for marker in ["client_id:\"", "\"client_id\":\"", "client_id=\""] {
        if let Some(pos) = js.find(marker) {
            let rest = &js[pos + marker.len()..];
            let id: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric())
                .collect();
            if id.len() >= 16 {
                return Some(id);
            }
        }
    }
    None
}

/// Bump a SoundCloud artwork URL (".../…-large.jpg", 100px) up to a
/// display-friendly 500px. The CDN honours the `-t500x500` size token, so this
/// is a plain string swap — falls back to the original URL otherwise.
fn upscale_artwork(url: &str) -> String {
    url.replace("-large.", "-t500x500.")
}

/// Hex-encode a remote artwork URL into the `urlhex_<hex>` tag the shared cover
/// resolver decodes. Inlined here for the same reason `ytmusic` inlines its own
/// copy: the `server` crate doesn't depend on `utils`.
fn encode_cover_url(url: &str) -> String {
    format!("urlhex_{}", hex::encode(url.as_bytes()))
}

/// Search the SoundCloud catalog for tracks matching `query`.
#[tracing::instrument(name = "soundcloud.search", fields(query = %query))]
pub async fn search_tracks(query: &str) -> Result<Vec<Track>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let http = http_client();

    let resp = match api_search(&http, query, &client_id(&http, false).await?).await {
        Ok(v) => v,
        Err(_) => api_search(&http, query, &client_id(&http, true).await?).await?,
    };

    let collection = resp
        .get("collection")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut out = Vec::with_capacity(collection.len());
    let mut seen = HashSet::new();
    for item in &collection {
        if item.get("kind").and_then(|v| v.as_str()) != Some("track") {
            continue;
        }
        if let Some(track) = parse_track(item) {
            let id = track_id(&track);
            if !id.is_empty() && seen.insert(id) {
                out.push(track);
            }
        }
    }
    Ok(out)
}

async fn api_search(http: &reqwest::Client, query: &str, cid: &str) -> Result<Value, String> {
    http.get(format!("{API_V2}/search/tracks"))
        .query(&[("q", query), ("client_id", cid), ("limit", "50")])
        .send()
        .await
        .map_err(|e| format!("SoundCloud search HTTP: {e}"))?
        .error_for_status()
        .map_err(|e| format!("SoundCloud search HTTP: {e}"))?
        .json::<Value>()
        .await
        .map_err(|e| format!("SoundCloud search JSON: {e}"))
}

/// Resolve a track id to a playable progressive MP3 stream URL via the `/tracks`
/// lookup endpoint. Called lazily at play time (the search results don't need to
/// round-trip every stream URL through the queue).
#[tracing::instrument(name = "soundcloud.resolve_stream", fields(track_id = %track_id))]
pub async fn resolve_stream(track_id: &str) -> Result<String, String> {
    let http = http_client();

    let track = match lookup_track(&http, track_id, &client_id(&http, false).await?).await {
        Ok(v) => v,
        Err(_) => lookup_track(&http, track_id, &client_id(&http, true).await?).await?,
    };

    let transcodings = track
        .get("media")
        .and_then(|m| m.get("transcodings"))
        .and_then(|t| t.as_array())
        .ok_or("SoundCloud track exposes no media transcodings")?;

    let progressive_url = transcodings
        .iter()
        .find(|tc| transcoding_protocol(tc) == Some("progressive"))
        .and_then(|tc| tc.get("url"))
        .and_then(|v| v.as_str())
        .ok_or("SoundCloud track has no progressive (non-HLS) stream")?;

    let track_auth = track.get("track_authorization").and_then(|v| v.as_str());
    let cid = client_id(&http, false).await?;

    let mut req = http.get(progressive_url).query(&[("client_id", cid.as_str())]);
    if let Some(auth) = track_auth {
        req = req.query(&[("track_authorization", auth)]);
    }

    let resolved = req
        .send()
        .await
        .map_err(|e| format!("SoundCloud stream resolve HTTP: {e}"))?
        .error_for_status()
        .map_err(|e| format!("SoundCloud stream resolve HTTP: {e}"))?
        .json::<Value>()
        .await
        .map_err(|e| format!("SoundCloud stream resolve JSON: {e}"))?;

    resolved
        .get("url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| "SoundCloud returned no stream URL for this track".to_string())
}

async fn lookup_track(http: &reqwest::Client, id: &str, cid: &str) -> Result<Value, String> {
    http.get(format!("{API_V2}/tracks/{id}"))
        .query(&[("client_id", cid)])
        .send()
        .await
        .map_err(|e| format!("SoundCloud lookup HTTP: {e}"))?
        .error_for_status()
        .map_err(|e| format!("SoundCloud lookup HTTP: {e}"))?
        .json::<Value>()
        .await
        .map_err(|e| format!("SoundCloud lookup JSON: {e}"))
}

fn transcoding_protocol(tc: &Value) -> Option<&str> {
    tc.get("format")
        .and_then(|f| f.get("protocol"))
        .and_then(|p| p.as_str())
}

/// Pull the id segment out of a `soundcloud:<id>[:tag]` track path.
fn track_id(t: &Track) -> String {
    t.path
        .to_string_lossy()
        .split(':')
        .nth(1)
        .unwrap_or("")
        .to_string()
}

fn parse_track(item: &Value) -> Option<Track> {
    let track_id = item.get("id").and_then(|v| v.as_u64())?;

    let has_progressive = item
        .get("media")
        .and_then(|m| m.get("transcodings"))
        .and_then(|t| t.as_array())
        .is_some_and(|arr| {
            arr.iter()
                .any(|tc| transcoding_protocol(tc) == Some("progressive"))
        });
    if !has_progressive {
        return None;
    }

    let title = item
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let artist = item
        .get("user")
        .and_then(|u| u.get("username"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let artwork = item
        .get("artwork_url")
        .and_then(|v| v.as_str())
        .or_else(|| {
            item.get("user")
                .and_then(|u| u.get("avatar_url"))
                .and_then(|v| v.as_str())
        })
        .filter(|s| !s.is_empty())
        .map(upscale_artwork);

    let duration = item
        .get("full_duration")
        .and_then(|v| v.as_u64())
        .or_else(|| item.get("duration").and_then(|v| v.as_u64()))
        .map(|ms| ms / 1000)
        .unwrap_or(0);

    let cover_tag = artwork.as_ref().map(|url| encode_cover_url(url));
    let path = match &cover_tag {
        Some(tag) => PathBuf::from(format!("{SOURCE_PREFIX}:{track_id}:{tag}")),
        None => PathBuf::from(format!("{SOURCE_PREFIX}:{track_id}")),
    };
    let album_id = match &cover_tag {
        Some(tag) => format!("{SOURCE_PREFIX}:_:{tag}"),
        None => format!("{SOURCE_PREFIX}:_"),
    };

    Some(Track {
        path,
        album_id,
        title,
        artist: artist.clone(),
        album: String::new(),
        duration,
        khz: 0,
        bitrate: 0,
        track_number: None,
        disc_number: None,
        musicbrainz_release_id: None,
        musicbrainz_recording_id: None,
        musicbrainz_track_id: None,
        playlist_item_id: None,
        artists: if artist.is_empty() {
            Vec::new()
        } else {
            vec![artist]
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_client_id_from_bundle_forms() {
        assert_eq!(
            find_client_id(r#"foo,client_id:"abcdefghij0123456789",bar"#).as_deref(),
            Some("abcdefghij0123456789")
        );
        assert_eq!(
            find_client_id(r#"{"client_id":"ABCDEFGHIJ0123456789"}"#).as_deref(),
            Some("ABCDEFGHIJ0123456789")
        );
        assert_eq!(find_client_id(r#"client_id:"short""#), None);
        assert_eq!(find_client_id("no id here"), None);
    }

    #[test]
    fn extract_attr_reads_src() {
        let chunk = r#" crossorigin src="https://a-v2.sndcdn.com/assets/0-abc.js"></script>"#;
        assert_eq!(
            extract_attr(chunk, "src"),
            Some("https://a-v2.sndcdn.com/assets/0-abc.js")
        );
        assert_eq!(extract_attr("<div>", "src"), None);
    }

    #[test]
    fn upscale_artwork_swaps_size_token() {
        assert_eq!(
            upscale_artwork("https://i1.sndcdn.com/artworks-xyz-large.jpg"),
            "https://i1.sndcdn.com/artworks-xyz-t500x500.jpg"
        );
        assert_eq!(upscale_artwork("https://x/y.png"), "https://x/y.png");
    }

    #[test]
    fn parse_track_requires_progressive_transcoding() {
        let hls_only = serde_json::json!({
            "id": 1, "title": "t", "kind": "track",
            "user": {"username": "u"},
            "media": {"transcodings": [{"url": "x", "format": {"protocol": "hls"}}]}
        });
        assert!(parse_track(&hls_only).is_none());

        let ok = serde_json::json!({
            "id": 42, "title": "Song", "kind": "track",
            "duration": 215000,
            "artwork_url": "https://i1.sndcdn.com/artworks-z-large.jpg",
            "user": {"username": "Artist"},
            "media": {"transcodings": [{"url": "x", "format": {"protocol": "progressive"}}]}
        });
        let t = parse_track(&ok).expect("progressive track parses");
        assert_eq!(t.title, "Song");
        assert_eq!(t.artist, "Artist");
        assert_eq!(t.duration, 215);
        assert_eq!(track_id(&t), "42");
        assert!(
            t.path
                .to_string_lossy()
                .starts_with("soundcloud:42:urlhex_")
        );
    }
}
