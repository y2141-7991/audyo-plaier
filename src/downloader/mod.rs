use std::path::Path;

use crate::downloader::{
    client::YoutubeClient,
    media_downloader::{Downloader, generate_filename},
};

pub mod client;
mod constant;
pub mod facade;
pub mod media_downloader;

pub struct YoutubeFacade {
    client: YoutubeClient,
    downloader: Downloader,
}

// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let ytb_client = YoutubeClient::default_android();
//     let a = ytb_client.get_video_info("7e8nNMOyQKU").await?;
//     let downloader = Downloader::new();
//     let filename = generate_filename(&a.title, "m4a");

//     let output_dir = Path::new("/output_dir");
//     let output_path = output_dir.join(filename);
//     for format in a.formats {
//         downloader.download(&format, &output_path).await?;
//     }

//     Ok(())
// }
