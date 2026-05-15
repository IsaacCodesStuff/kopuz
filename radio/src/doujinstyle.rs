use crate::{RadioMetadata, RadioMetadataProvider};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Debug)]
struct DoujinstyleResponse {
    data: DoujinstyleData,
}

#[derive(Deserialize, Debug)]
struct DoujinstyleData {
    track_artist: Option<String>,
    track_title: Option<String>,
    title: Option<String>, // fallback if track_title is missing
    artwork_urls: Option<DoujinstyleArtwork>,
}

#[derive(Deserialize, Debug)]
struct DoujinstyleArtwork {
    large: Option<String>,
}

pub struct DoujinstyleProvider;

impl RadioMetadataProvider for DoujinstyleProvider {
    fn start(&self, _stream_id: &str) -> mpsc::UnboundedReceiver<RadioMetadata> {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent("Kopuz/0.5.5")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            let mut last_title = String::new();

            loop {
                let req = client.get("https://public.radio.co/api/v2/s5ff57669c/track/current")
                    .send()
                    .await;

                if let Ok(resp) = req {
                    if let Ok(json) = resp.json::<DoujinstyleResponse>().await {
                        let title = json.data.track_title.unwrap_or_else(|| json.data.title.unwrap_or_default());
                        let artist = json.data.track_artist.unwrap_or_default();
                        let cover_url = json.data.artwork_urls.and_then(|a| a.large);

                        let comparison_str = format!("{} - {}", artist, title);
                        if comparison_str != last_title {
                            last_title = comparison_str;

                            let meta = RadioMetadata {
                                title,
                                artist,
                                cover_url,
                            };

                            if tx.send(meta).is_err() {
                                break;
                            }
                        }
                    }
                }

                sleep(Duration::from_secs(5)).await;
            }
        });

        rx
    }
}
