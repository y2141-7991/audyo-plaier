use async_trait::async_trait;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Debug)]
struct ClientConfig {
    client_name: String,
    client_version: String,
    user_agent: String,
    language: String,
    country: String,
    extra_config: Value,
}

#[async_trait]
trait ClientStrategy: Send + Sync {
    fn config(&self) -> &ClientConfig;
    fn build_payload(&self, video_id: &str) -> Value;
    fn build_headers(&self, base_url: &'static str) -> HeaderMap;
    fn client_name(&self) -> &str;
    fn client_number(&self) -> u32;
}

pub struct AndroidClient {
    client_config: ClientConfig,
}

impl AndroidClient {
    fn new() -> Self {
        Self {
            client_config: ClientConfig {
                client_name: "ANDROID".to_string(),
                client_version: "19.01.34".to_string(),
                user_agent: "com.google.android.youtube/19.01.34 (Linux; Android 13)".to_string(),
                language: "en".to_string(),
                country: "US".to_string(),
                extra_config: json!({
                    "androidSdkVersion": 33,
                    "osName": "Android",
                    "osVersion": "13",
                    "platform": "MOBILE",
                    "deviceMake": "Google",
                    "deviceModel": "Pixel 7"
                }),
            },
        }
    }
}

impl Default for AndroidClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientStrategy for AndroidClient {
    fn config(&self) -> &ClientConfig {
        &self.client_config
    }
    fn build_payload(&self, video_id: &str) -> Value {
        json!({
            "videoId": video_id,
            "context": {
                "client": {
                    "hl": self.client_config.language,
                    "gl": self.client_config.country,
                    "clientName": self.client_config.client_name,
                    "clientVersion": self.client_config.client_version,
                    "androidSdkVersion": self.client_config.extra_config["androidSdkVersion"],
                    "osName": self.client_config.extra_config["osName"],
                    "osVersion": self.client_config.extra_config["osVersion"],
                    "platform": self.client_config.extra_config["platform"],
                    "deviceMake": self.client_config.extra_config["deviceMake"],
                    "deviceModel": self.client_config.extra_config["deviceMake"],
                    "userAgent": self.client_config.user_agent,
                },
                "user": {
                    "lockedSafetyMode": false
                },
                "request": {
                    "useSsl": true
                }
            },
            "playbackContext": {
                "contentPlaybackContext": {
                    "signatureTimestamp": 20438
                }
            }
        })
    }
    fn build_headers(&self, base_url: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(&self.client_config.user_agent).unwrap(),
        );
        headers.insert("Accept", HeaderValue::from_static("*/*"));
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("X-Youtube-Client-Name", HeaderValue::from_static("3"));
        headers.insert(
            "X-Youtube-Client-Version",
            HeaderValue::from_str(&self.client_config.client_version).unwrap(),
        );
        headers.insert("Origin", HeaderValue::from_static(base_url));
        headers.insert(
            "Referer",
            HeaderValue::from_static("https://www.youtube.com/"),
        );
        headers
    }
    fn client_name(&self) -> &str {
        "ANDROID"
    }
    fn client_number(&self) -> u32 {
        3
    }
}

pub struct YoutubeClient {
    http: Client,
    strategy: Arc<dyn ClientStrategy>,
}

impl YoutubeClient {
    const API_YOUTUBE_URL: &'static str = "https://www.youtube.com/youtubei/v1/player";
    const YOUTUBE_URL: &'static str = "https://www.youtube.com";

    fn new(strategy: Arc<dyn ClientStrategy>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        YoutubeClient {
            http: client,
            strategy: strategy,
        }
    }
    pub fn default_android() -> Self {
        Self::new(Arc::new(AndroidClient::new()))
    }
    fn set_strategy(&mut self, strategy: Arc<dyn ClientStrategy>) {
        self.strategy = strategy
    }
    pub async fn get_video_info(&self, video_id: &str) -> Result<VideoInfo> {
        let headers = self.strategy.build_headers(Self::YOUTUBE_URL);
        let payload = self.strategy.build_payload(video_id);

        let response = self
            .http
            .post(Self::API_YOUTUBE_URL)
            .query(&[("prettyPrint", "false")])
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        response.error_for_status_ref()?;
        let player_response = response.json::<PlayerResponse>().await?;

        self.parse_player_payload(player_response, video_id)
    }

    fn parse_player_payload(
        &self,
        player_response: PlayerResponse,
        video_id: &str,
    ) -> Result<VideoInfo> {
        let details = player_response
            .video_details
            .ok_or_else(|| YtdlError::VideoNotFound(video_id.to_string()))?;
        let streaming = player_response
            .streaming_data
            .ok_or(YtdlError::NoSuitableFormat)?;
        let mut formats = Vec::new();
        if let Some(regular) = streaming.formats {
            formats.extend(regular);
        }
        Ok(VideoInfo {
            video_id: details.video_id,
            title: details.title,
            author: details.author,
            length_seconds: details.length_seconds.parse::<u32>().unwrap_or(0),
            formats: formats,
        })
    }
    pub fn extract_video_id(url: &str) -> Option<String> {
        let patterns = [
            (r"youtube\.com/watch\?.*v=([a-zA-Z0-9_-]{11})", 1),
            (r"youtu\.be/([a-zA-Z0-9_-]{11})", 1),
            (r"youtube\.com/embed/([a-zA-Z0-9_-]{11})", 1),
            (r"youtube\.com/v/([a-zA-Z0-9_-]{11})", 1),
            (r"youtube\.com/shorts/([a-zA-Z0-9_-]{11})", 1),
        ];

        for (pattern, group) in patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if let Some(caps) = re.captures(url) {
                    if let Some(m) = caps.get(group) {
                        return Some(m.as_str().to_string());
                    }
                }
            }
        }
        None
    }
}

#[derive(Deserialize, Debug)]
pub struct VideoInfo {
    video_id: String,
    title: String,
    author: String,
    length_seconds: u32,
    formats: Vec<FormatResponse>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PlayerResponse {
    video_details: Option<VideoDetails>,
    streaming_data: Option<StreamingData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VideoDetails {
    video_id: String,
    author: String,
    length_seconds: String,
    title: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct StreamingData {
    formats: Option<Vec<FormatResponse>>,
    expires_in_seconds: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FormatResponse {
    itag: Option<u32>,
    url: Option<String>,
    approx_duration_ms: Option<String>,
    audio_channels: Option<u8>,
    audio_quality: Option<String>,
    audio_sample_rate: Option<String>,
    average_bitrate: Option<u32>,
    bitrate: Option<u32>,
    mime_type: Option<String>,
    quality: Option<String>,
}

#[derive(Error, Debug)]
pub enum YtdlError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Video not found: {0}")]
    VideoNotFound(String),

    #[error("Format not available: itag={0}")]
    FormatNotAvailable(u32),

    #[error("No suitable format found")]
    NoSuitableFormat,

    #[error("Playability error: {status} - {reason}")]
    PlayabilityError { status: String, reason: String },

    #[error("Signature decryption required (not implemented)")]
    SignatureRequired,

    #[error("Rate limited by YouTube")]
    RateLimited,

    #[error("Invalid video ID: {0}")]
    InvalidVideoId(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, YtdlError>;
