use crate::error::IpcError;
use crate::video::{self, VideoInfo};
use crate::video::downloader::DownloadManager;
use std::path::PathBuf;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn parse_video_url(url: String) -> Result<VideoInfo, IpcError> {
    video::parse_video_url(&url).await
}

#[tauri::command]
pub async fn download_video(
    app: AppHandle,
    manager: State<'_, DownloadManager>,
    task_id: String,
    title: String,
    video_url: String,
    audio_url: Option<String>,
    quality: String,
    save_dir: Option<String>,
) -> Result<String, IpcError> {
    let save_dir = if let Some(dir) = save_dir {
        PathBuf::from(dir)
    } else {
        dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
            .ok_or_else(|| IpcError::internal("Cannot determine download directory"))?
    };

    let format = video::VideoFormat {
        quality,
        url: video_url,
        audio_url,
        size: None,
    };

    let save_path = manager
        .start_download(app, task_id, title, format, save_dir)
        .await?;

    Ok(save_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn cancel_video_download(
    manager: State<'_, DownloadManager>,
    task_id: String,
) -> Result<(), IpcError> {
    manager.cancel_download(&task_id).await
}

#[tauri::command]
pub async fn open_in_folder(path: String) -> Result<(), IpcError> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(IpcError::not_found("File not found"));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| IpcError::internal(e.to_string()))?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| IpcError::internal(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path.parent().unwrap_or(&path))
            .spawn()
            .map_err(|e| IpcError::internal(e.to_string()))?;
    }

    Ok(())
}
