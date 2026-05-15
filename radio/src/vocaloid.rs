use crate::{RadioMetadata, RadioMetadataProvider};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Debug)]
struct VocaloidResponse {
    #[serde(rename = "Primary")]
    primary: Option<VocaloidPrimary>,
}

#[derive(Deserialize, Debug)]
struct VocaloidPrimary {
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Subtitle")]
    subtitle: Option<String>,
    #[serde(rename = "Image")]
    image: Option<String>,
}

pub struct VocaloidProvider;

impl RadioMetadataProvider for VocaloidProvider {
    fn start(&self, _stream_id: &str) -> mpsc::UnboundedReceiver<RadioMetadata> {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .user_agent("Kopuz/0.5.5")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            let mut last_title = String::new();

            loop {
                let req = client.get("https://feed.platform.prod.us-west-2.tunein.com/profiles/s221579/nowPlaying")
                    .send()
                    .await;

                if let Ok(resp) = req {
                    if let Ok(json) = resp.json::<VocaloidResponse>().await {
                        if let Some(primary) = json.primary {
                            let title = primary.title.unwrap_or_default();
                            let artist = primary.subtitle.unwrap_or_default();
                            let cover_url = primary.image;

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
                }

                sleep(Duration::from_secs(7)).await;
            }
        });

        rx
    }
}
