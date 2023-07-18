use std::path::PathBuf;
use directories::ProjectDirs;

pub(crate) fn get_cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "tudders", "seph").map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}
