use async_trait::async_trait;
use serde_json::Value;
use reqwest::header::HeaderMap;


#[derive(Clone, Debug)]
struct ClientConfig {
    client_name: String,
    client_version: String,
    user_agent: String,
    language: String,
    country: String,
    extra_config : Value,
}

#[async_trait]
trait ClientStrategy: Send + Sync {
    fn config(&self) -> ClientConfig;
    fn build_payload(&self) -> Value;
    fn build_headers(&self) -> HeaderMap;
    fn client_name(&self) -> &str;
    fn client_number(&self) -> u32;
}