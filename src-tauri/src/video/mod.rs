pub mod bilibili;
pub mod douyin;
pub mod downloader;

use crate::error::IpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub cover_url: Option<String>,
    pub duration: Option<u64>,
    pub platform: String,
    pub formats: Vec<VideoFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFormat {
    pub quality: String,
    pub url: String,
    pub audio_url: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub task_id: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: u64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Downloading,
    Completed,
    Failed(String),
    Cancelled,
}

#[async_trait::async_trait]
pub trait VideoParser: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn parse(&self, url: &str, client: &reqwest::Client) -> Result<VideoInfo, IpcError>;
}

pub async fn parse_video_url(input: &str) -> Result<VideoInfo, IpcError> {
    // Extract URL from share text (e.g. Douyin share messages contain URL mixed with text)
    let url = extract_url_from_text(input).unwrap_or_else(|| input.trim().to_string());

    let client = create_http_client();
    let parsers: Vec<Box<dyn VideoParser>> = vec![
        Box::new(douyin::DouyinParser),
        Box::new(bilibili::BilibiliParser),
    ];
    for parser in &parsers {
        if parser.can_handle(&url) {
            return parser.parse(&url, &client).await;
        }
    }
    Err(IpcError::validation(
        "Unsupported video URL. Currently supported: Douyin, Bilibili",
    ))
}

/// Extract the first HTTP(S) URL from a block of text.
fn extract_url_from_text(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"https?://[^\s\u{4e00}-\u{9fff}\u{ff01}-\u{ff5e}]+").ok()?;
    re.find(text).map(|m| m.as_str().trim_end_matches(|c: char| ",.;:!?，。；：！？)）】》".contains(c)).to_string())
}

pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .expect("failed to build http client")
}
