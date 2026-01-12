use std::path::PathBuf;

use crate::downloader::{
    client::{Result, YoutubeClient},
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
        for format in video_info.formats {
            self.downloader.download(&format, output_path).await?;
        }
        Ok(())
    }
    pub fn extract_video_id_from_url(&self, url: &str) -> Option<String> {
        YoutubeClient::extract_video_id(url)
    }
}
