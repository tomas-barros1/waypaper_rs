use gtk::gdk_pixbuf::Pixbuf;
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

pub struct ThumbnailCache;

impl ThumbnailCache {
    pub fn png(path: &Path, width: i32, height: i32) -> Option<Vec<u8>> {
        let cache_path = Self::path(path, width, height)?;
        if let Ok(bytes) = fs::read(&cache_path) {
            return Some(bytes);
        }
        let pixbuf = Pixbuf::from_file_at_scale(path, width, height, true).ok()?;
        let bytes = pixbuf.save_to_bufferv("png", &[]).ok()?;
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(cache_path, &bytes);
        Some(bytes)
    }

    fn path(path: &Path, width: i32, height: i32) -> Option<PathBuf> {
        let metadata = fs::metadata(path).ok()?;
        let modified = metadata
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?;
        let mut hasher = DefaultHasher::new();
        path.to_string_lossy().hash(&mut hasher);
        metadata.len().hash(&mut hasher);
        modified.as_secs().hash(&mut hasher);
        modified.subsec_nanos().hash(&mut hasher);
        width.hash(&mut hasher);
        height.hash(&mut hasher);
        Some(
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("waypaper-rs")
                .join("thumbnails")
                .join(format!("{:016x}.png", hasher.finish())),
        )
    }
}
