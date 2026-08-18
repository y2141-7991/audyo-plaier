use std::path::PathBuf;

use crate::downloader::{
    client::{Result, YoutubeClient, YtdlError},
    media_downloader::{Downloader, generate_filename},
};

pub struct YoutubeFacade {
    client: YoutubeClient,
    downloader: Downloader,
    pub output_dir: PathBuf,
}

impl YoutubeFacade {
    pub fn new() -> Self {
        let ytb_client = YoutubeClient::default_android();
        let downloader = Downloader::new();
        let output_dir = if let Some(home) = dirs::home_dir() {
            home.join(".audyo_plaier")
        } else {
            PathBuf::from("./audyo_plaier")
        }
        .join("audio");

        Self {
            client: ytb_client,
            downloader: downloader,
            output_dir: output_dir,
        }
    }
    pub async fn download_audio(&self, video_id: &str) -> Result<()> {
        let video_info = self.client.get_video_info(video_id).await?;
        let filename = generate_filename(&video_info.title, "m4a");
        let output_path = &self.output_dir.join(filename);
        let format =
            select_best_format(video_info.formats).ok_or(YtdlError::NoSuitableFormat)?;
        self.downloader.download(&format, output_path).await?;
        Ok(())
    }
    pub fn extract_video_id_from_url(&self, url: &str) -> Option<String> {
        YoutubeClient::extract_video_id(url)
    }
}

/// Prefers muxed progressive streams, which decode reliably, over YouTube's
/// fragmented adaptive audio-only streams, which the app's MP4/WebM decoders
/// can't parse.
fn select_best_format(
    formats: Vec<crate::downloader::client::FormatResponse>,
) -> Option<crate::downloader::client::FormatResponse> {
    let mut progressive = Vec::new();
    let mut audio_only = Vec::new();
    for format in formats {
        if format.url.is_none() {
            continue;
        }
        if format.is_progressive_audio() {
            progressive.push(format);
        } else if format.is_audio() {
            audio_only.push(format);
        }
    }
    progressive
        .into_iter()
        .max_by_key(|f| f.bitrate())
        .or_else(|| audio_only.into_iter().max_by_key(|f| f.bitrate()))
}
