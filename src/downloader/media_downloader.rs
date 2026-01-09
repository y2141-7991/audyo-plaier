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
    download_config: DownloaderConfig
}

impl DownloadBuilder {
    fn new() {}
}