
use std::path::{Path, PathBuf};

use reqwest::{Client, header::{HeaderMap, HeaderValue, REFERER, USER_AGENT}};
use futures_util::StreamExt;
use tokio::fs::File;
use crate::downloader::client::{FormatResponse, Result, YtdlError};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
struct DownloaderConfig {
    chunked_size: usize,
    user_agent: String,
    max_retries: u32,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        DownloaderConfig {
            chunked_size: 1024,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            max_retries: 3,
        }
    }
}

struct DownloadBuilder {
    download_config: DownloaderConfig,
}

impl DownloadBuilder {
    fn new() -> Self {
        DownloadBuilder {
            download_config: DownloaderConfig::default(),
        }
    }
    fn chunk_size(mut self, chunk_size: usize) -> Self {
        self.download_config.chunked_size = chunk_size;
        self
    }
    fn max_retries(mut self, max_retries: u32) -> Self {
        self.download_config.max_retries = max_retries;
        self
    }
    fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.download_config.user_agent = user_agent.into();
        self
    }
}

pub struct Downloader {
    client: Client,
    download_config: DownloaderConfig,
}

impl Downloader {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        let download_config = DownloaderConfig::default();
        Downloader {
            client,
            download_config,
        }
    }
    pub async fn download(&self, format_response: &FormatResponse, output_path: &Path) -> Result<PathBuf> {
        let url = format_response.url.as_ref().ok_or(YtdlError::FormatNotAvailable(18))?; 
        self.download_url(url, output_path).await
    }
    async fn download_url(&self, url: &str, output_path: &Path) -> Result<PathBuf>{
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&self.download_config.user_agent).unwrap());
        headers.insert(REFERER, HeaderValue::from_static("https://www.youtube.com"));

        let response = self.client.get(url).headers(headers).send().await?;


        response.error_for_status_ref()?;
        let mut file = File::create(output_path).await?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;

        Ok(output_path.to_path_buf())
    }
}


fn sanitize_filename(filename: &str) -> String {
    filename.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn generate_filename(filename: &str, extension: &str) -> String {
    format!("{}.{}", sanitize_filename(filename), extension)
}