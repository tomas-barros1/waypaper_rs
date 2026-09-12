use super::cache::Cache;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WallpaperError {
    #[error("no wallpaper has been selected")]
    NoWallpaper,
    #[error("folder does not exist: {0}")]
    InvalidFolder(String),
    #[error("no supported wallpaper backend found (install swaybg or hyprpaper)")]
    NoBackend,
    #[error("{0} failed: {1}")]
    CommandFailed(String, String),
    #[error("could not save cache: {0}")]
    Cache(String),
}

pub struct WallpaperService {
    pub cache: Cache,
}

impl WallpaperService {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }
    pub fn wallpapers_in(&self, folder: &Path) -> Vec<PathBuf> {
        let mut files: Vec<_> = std::fs::read_dir(folder)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && Self::is_image(path))
            .collect();
        files.sort();
        files
    }
    pub fn set_folder(&self, folder: PathBuf) -> Result<(), WallpaperError> {
        if !folder.is_dir() {
            return Err(WallpaperError::InvalidFolder(folder.display().to_string()));
        }
        let mut cache = self.cache.clone();
        cache.folder = Some(folder);
        cache.save().map_err(WallpaperError::Cache)
    }
    pub fn set_wallpaper(&self, wallpaper: PathBuf) -> Result<(), WallpaperError> {
        if !wallpaper.is_file() {
            return Err(WallpaperError::NoWallpaper);
        }
        let backend = self.backend()?;
        match backend.as_str() {
            "hyprpaper" => self.hyprpaper(&wallpaper)?,
            "swaybg" => self.swaybg(&wallpaper)?,
            _ => return Err(WallpaperError::NoBackend),
        }
        let mut cache = self.cache.clone();
        cache.wallpaper = Some(wallpaper);
        if let Some(wallpaper) = &cache.wallpaper {
            if let Some(folder) = wallpaper.parent() {
                cache.folder = Some(folder.to_path_buf());
            }
        }
        cache.backend = Some(backend);
        cache.swaybg_pid = None;
        cache.save().map_err(WallpaperError::Cache)
    }
    pub fn restore(&self) -> Result<(), WallpaperError> {
        self.set_wallpaper(
            self.cache
                .wallpaper
                .clone()
                .ok_or(WallpaperError::NoWallpaper)?,
        )
    }
    fn backend(&self) -> Result<String, WallpaperError> {
        if let Some(name) = &self.cache.backend {
            if Self::available(name) {
                return Ok(name.clone());
            }
        }
        if Self::available("hyprpaper") {
            Ok("hyprpaper".into())
        } else if Self::available("swaybg") {
            Ok("swaybg".into())
        } else {
            Err(WallpaperError::NoBackend)
        }
    }
    fn available(name: &str) -> bool {
        Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {name}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    fn hyprpaper(&self, path: &Path) -> Result<(), WallpaperError> {
        self.run(
            "hyprctl",
            &["hyprpaper", "preload", &path.to_string_lossy()],
        )?;
        self.run(
            "hyprctl",
            &["hyprpaper", "wallpaper", &format!(",{}", path.display())],
        )
    }
    fn swaybg(&self, path: &Path) -> Result<(), WallpaperError> {
        if let Some(pid) = self.cache.swaybg_pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
        let child = Command::new("swaybg")
            .args(["-i", &path.to_string_lossy(), "-m", "fill"])
            .spawn()
            .map_err(|e| WallpaperError::CommandFailed("swaybg".into(), e.to_string()))?;
        let mut cache = self.cache.clone();
        cache.swaybg_pid = Some(child.id());
        cache.wallpaper = Some(path.to_path_buf());
        cache.backend = Some("swaybg".into());
        cache.save().map_err(WallpaperError::Cache)
    }
    fn run(&self, command: &str, args: &[&str]) -> Result<(), WallpaperError> {
        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|e| WallpaperError::CommandFailed(command.into(), e.to_string()))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(WallpaperError::CommandFailed(
                command.into(),
                String::from_utf8_lossy(&output.stderr).trim().into(),
            ))
        }
    }
    fn is_image(path: &Path) -> bool {
        matches!(
            path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .as_deref(),
            Some("jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif" | "avif")
        )
    }
}
