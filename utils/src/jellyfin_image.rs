pub fn jellyfin_image_url(
    server_url: &str,
    item_id: &str,
    image_tag: Option<&str>,
    access_token: Option<&str>,
    max_width: u32,
    quality: u32,
) -> String {
    let mut url = format!(
        "{}/Items/{}/Images/Primary?maxWidth={}&quality={}",
        server_url, item_id, max_width, quality
    );
    if let Some(tag) = image_tag {
        url.push_str(&format!("&tag={}", tag));
    }
    if let Some(token) = access_token {
        url.push_str(&format!("&api_key={}", token));
    }
    url
}

pub fn parse_jellyfin_path(path_str: &str) -> Option<(&str, Option<&str>)> {
    let parts: Vec<&str> = path_str.split(':').collect();
    if parts.len() >= 2 {
        let id = parts[1];
        let tag = if parts.len() >= 3 {
            Some(parts[2])
        } else {
            None
        };
        Some((id, tag))
    } else {
        None
    }
}

pub fn jellyfin_image_url_from_path(
    path_str: &str,
    server_url: &str,
    access_token: Option<&str>,
    max_width: u32,
    quality: u32,
) -> Option<String> {
    let (id, tag) = parse_jellyfin_path(path_str)?;
    Some(jellyfin_image_url(
        server_url,
        id,
        tag,
        access_token,
        max_width,
        quality,
    ))
}
