use super::{VideoFormat, VideoInfo, VideoParser};
use crate::error::IpcError;
use regex::Regex;
use std::collections::HashSet;

pub struct BilibiliParser;

#[async_trait::async_trait]
impl VideoParser for BilibiliParser {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("bilibili.com") || url.contains("b23.tv")
    }

    async fn parse(&self, url: &str, client: &reqwest::Client) -> Result<VideoInfo, IpcError> {
        // Resolve short URL
        let resolved_url = if url.contains("b23.tv") {
            resolve_short_url(url).await?
        } else {
            url.to_string()
        };

        // Extract BV ID
        let bvid = extract_bvid(&resolved_url)?;

        // Fetch view info
        let view_url = format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        );

        let view_resp: serde_json::Value = client
            .get(&view_url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?
            .json()
            .await?;

        let data = view_resp
            .get("data")
            .ok_or_else(|| IpcError::internal("Failed to get video info from Bilibili API"))?;

        let title = data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Bilibili Video")
            .to_string();

        let cover_url = data
            .get("pic")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let duration = data.get("duration").and_then(|v| v.as_u64());

        let cid = data
            .get("cid")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| IpcError::internal("Failed to get cid from Bilibili video info"))?;

        // Fetch play URLs (DASH format)
        let play_url = format!(
            "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&fnval=16&fourk=1",
            bvid, cid
        );

        let play_resp: serde_json::Value = client
            .get(&play_url)
            .header("Referer", "https://www.bilibili.com/")
            .send()
            .await?
            .json()
            .await?;

        let play_data = play_resp
            .get("data")
            .ok_or_else(|| IpcError::internal("Failed to get play URL data from Bilibili API"))?;

        let mut formats = Vec::new();

        // Try DASH format first
        if let Some(dash) = play_data.get("dash") {
            let audio_url = dash
                .pointer("/audio/0/baseUrl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let Some(videos) = dash.get("video").and_then(|v| v.as_array()) {
                let mut seen_qualities = HashSet::new();

                for video in videos {
                    let quality_id = video.get("id").and_then(|v| v.as_u64()).unwrap_or(0);

                    // Deduplicate by quality level
                    if !seen_qualities.insert(quality_id) {
                        continue;
                    }

                    let quality_label = quality_label(quality_id);

                    if let Some(base_url) = video.get("baseUrl").and_then(|v| v.as_str()) {
                        formats.push(VideoFormat {
                            quality: quality_label,
                            url: base_url.to_string(),
                            audio_url: audio_url.clone(),
                            size: None,
                        });
                    }
                }
            }
        }

        // Fallback to durl format
        if formats.is_empty() {
            if let Some(durls) = play_data.get("durl").and_then(|v| v.as_array()) {
                for durl in durls {
                    if let Some(url) = durl.get("url").and_then(|v| v.as_str()) {
                        let size = durl.get("size").and_then(|v| v.as_u64());
                        formats.push(VideoFormat {
                            quality: "default".to_string(),
                            url: url.to_string(),
                            audio_url: None,
                            size,
                        });
                    }
                }
            }
        }

        if formats.is_empty() {
            return Err(IpcError::internal("No playable video formats found"));
        }

        Ok(VideoInfo {
            title,
            cover_url,
            duration,
            platform: "bilibili".to_string(),
            formats,
        })
    }
}

async fn resolve_short_url(url: &str) -> Result<String, IpcError> {
    let no_redirect_client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| IpcError::internal(e.to_string()))?;

    let resp = no_redirect_client.get(url).send().await?;

    if let Some(location) = resp.headers().get("location") {
        let location = location
            .to_str()
            .map_err(|e| IpcError::internal(e.to_string()))?;
        Ok(location.to_string())
    } else {
        Err(IpcError::internal("Failed to resolve Bilibili short URL"))
    }
}

fn extract_bvid(url: &str) -> Result<String, IpcError> {
    let re =
        Regex::new(r"(BV[a-zA-Z0-9]+)").map_err(|e| IpcError::internal(e.to_string()))?;

    if let Some(caps) = re.captures(url) {
        if let Some(bvid) = caps.get(1) {
            return Ok(bvid.as_str().to_string());
        }
    }

    Err(IpcError::validation(
        "Could not extract BV ID from Bilibili URL",
    ))
}

fn quality_label(id: u64) -> String {
    match id {
        127 => "8K".to_string(),
        126 => "Dolby Vision".to_string(),
        125 => "HDR".to_string(),
        120 => "4K".to_string(),
        116 => "1080P60".to_string(),
        112 => "1080P+".to_string(),
        80 => "1080P".to_string(),
        74 => "720P60".to_string(),
        64 => "720P".to_string(),
        32 => "480P".to_string(),
        16 => "360P".to_string(),
        _ => format!("{}P", id),
    }
}
