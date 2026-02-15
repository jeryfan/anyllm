use super::{VideoFormat, VideoInfo, VideoParser};
use crate::error::IpcError;
use regex::Regex;
use serde_json::Value;

pub struct DouyinParser;

#[async_trait::async_trait]
impl VideoParser for DouyinParser {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("douyin.com") || url.contains("douyinvod.com") || url.contains("iesdouyin.com")
    }

    async fn parse(&self, url: &str, client: &reqwest::Client) -> Result<VideoInfo, IpcError> {
        // Direct CDN URL — return as-is
        if is_direct_cdn_url(url) {
            return Ok(VideoInfo {
                title: "Douyin Video".to_string(),
                cover_url: None,
                duration: None,
                platform: "douyin".to_string(),
                formats: vec![VideoFormat {
                    quality: "original".to_string(),
                    url: url.to_string(),
                    audio_url: None,
                    size: None,
                }],
            });
        }

        // Resolve short URL (v.douyin.com/xxx) to full page URL to extract video ID
        let page_url = if url.contains("v.douyin.com") {
            resolve_short_url(client, url).await?
        } else {
            url.to_string()
        };

        let video_id = extract_video_id(&page_url)?;

        // Use iesdouyin mobile share page — this returns _ROUTER_DATA with video info
        fetch_via_mobile_share(client, &video_id).await
    }
}

fn is_direct_cdn_url(url: &str) -> bool {
    (url.contains("douyinvod.com") && url.contains("/video/"))
        || url.contains("mime_type=video")
}

/// Follow redirects to get the final URL.
async fn resolve_short_url(client: &reqwest::Client, url: &str) -> Result<String, IpcError> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| IpcError::internal(format!("Failed to resolve short URL: {}", e)))?;
    Ok(resp.url().to_string())
}

/// Fetch video info from iesdouyin mobile share page.
/// The page embeds `_ROUTER_DATA = {...}` with full video details.
async fn fetch_via_mobile_share(
    client: &reqwest::Client,
    video_id: &str,
) -> Result<VideoInfo, IpcError> {
    let share_url = format!(
        "https://www.iesdouyin.com/share/video/{}",
        video_id
    );

    let mobile_ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

    let resp = client
        .get(&share_url)
        .header("User-Agent", mobile_ua)
        .header("Referer", "https://www.douyin.com/")
        .send()
        .await
        .map_err(|e| IpcError::internal(format!("Failed to fetch Douyin share page: {}", e)))?;

    let html = resp
        .text()
        .await
        .map_err(|e| IpcError::internal(format!("Failed to read share page: {}", e)))?;

    // Extract _ROUTER_DATA JSON from the page
    let router_data = extract_router_data(&html)?;

    // Navigate to the video item:
    // loaderData["video_(id)/page"].videoInfoRes.item_list[0]
    let item = router_data
        .pointer("/loaderData")
        .and_then(|ld| {
            // The key is dynamic like "video_(id)/page"
            if let Value::Object(map) = ld {
                for (key, val) in map {
                    if key.contains("video") && key.contains("page") {
                        return val.pointer("/videoInfoRes/item_list/0");
                    }
                }
            }
            None
        })
        .ok_or_else(|| IpcError::internal("Video info not found in Douyin share page data"))?;

    let title = item
        .get("desc")
        .and_then(|v| v.as_str())
        .unwrap_or("Douyin Video")
        .to_string();

    let cover_url = item
        .pointer("/video/cover/url_list/0")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let duration = item
        .pointer("/video/duration")
        .and_then(|v| v.as_u64())
        .map(|ms| ms / 1000);

    let mut formats = Vec::new();

    // Get the video URI for constructing multi-quality URLs
    let video_uri = item
        .pointer("/video/play_addr/uri")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if !video_uri.is_empty() {
        // Determine available qualities based on video dimensions
        let width = item.pointer("/video/width").and_then(|v| v.as_u64()).unwrap_or(0);
        let height = item.pointer("/video/height").and_then(|v| v.as_u64()).unwrap_or(0);
        let max_dim = width.max(height);

        // Generate quality options from highest to lowest.
        // The ratio parameter controls quality: "default" returns original, others are re-encoded.
        let mut qualities: Vec<(&str, &str)> = Vec::new();

        if max_dim >= 1080 {
            qualities.push(("1080p (original)", "default"));
            qualities.push(("1080p", "1080p"));
        } else if max_dim >= 720 {
            qualities.push(("720p (original)", "default"));
        }

        if max_dim >= 720 {
            qualities.push(("720p", "720p"));
        }

        qualities.push(("540p", "540p"));

        for (label, ratio) in &qualities {
            let url = format!(
                "https://aweme.snssdk.com/aweme/v1/play/?video_id={}&ratio={}&line=0",
                video_uri, ratio
            );
            formats.push(VideoFormat {
                quality: label.to_string(),
                url,
                audio_url: None,
                size: None,
            });
        }
    }

    // Fallback: use play_addr URL directly if URI was empty
    if formats.is_empty() {
        if let Some(play_url) = item
            .pointer("/video/play_addr/url_list/0")
            .and_then(|v| v.as_str())
        {
            let url = play_url.replace("playwm", "play");
            formats.push(VideoFormat {
                quality: "default".to_string(),
                url,
                audio_url: None,
                size: None,
            });
        }
    }

    if formats.is_empty() {
        return Err(IpcError::internal(
            "No playable video URLs found in Douyin share page",
        ));
    }

    Ok(VideoInfo {
        title,
        cover_url,
        duration,
        platform: "douyin".to_string(),
        formats,
    })
}

/// Extract `_ROUTER_DATA = {...}` JSON from the share page HTML.
fn extract_router_data(html: &str) -> Result<Value, IpcError> {
    let re = Regex::new(r"_ROUTER_DATA\s*=\s*(\{.+\})\s*</script>")
        .map_err(|e| IpcError::internal(e.to_string()))?;

    let caps = re.captures(html).ok_or_else(|| {
        IpcError::internal("Could not find _ROUTER_DATA in Douyin share page")
    })?;

    let json_str = &caps[1];
    serde_json::from_str(json_str)
        .map_err(|e| IpcError::internal(format!("Failed to parse _ROUTER_DATA: {}", e)))
}

fn extract_video_id(text: &str) -> Result<String, IpcError> {
    // Match /video/NNNN or /note/NNNN
    let re = Regex::new(r"/(?:video|note)/(\d+)")
        .map_err(|e| IpcError::internal(e.to_string()))?;
    if let Some(caps) = re.captures(text) {
        if let Some(id) = caps.get(1) {
            return Ok(id.as_str().to_string());
        }
    }

    // Also try modal_id query param
    let re2 = Regex::new(r"modal_id=(\d+)")
        .map_err(|e| IpcError::internal(e.to_string()))?;
    if let Some(caps) = re2.captures(text) {
        if let Some(id) = caps.get(1) {
            return Ok(id.as_str().to_string());
        }
    }

    Err(IpcError::validation(
        "Could not extract video ID from Douyin URL",
    ))
}
