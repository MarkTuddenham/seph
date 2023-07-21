use std::io::{Read, Write};
use std::{path::PathBuf, fs::File};
use directories::ProjectDirs;
use libseph::{JobId, Job};

pub(crate) fn get_cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "tudders", "seph").map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

pub(crate) fn write_last_ran_job(job: &Job) {
    if let Some(path) = get_cache_dir() {
        let path = path.join("last_ran");
        let mut file = File::create(path).unwrap();
        file.write_all(job.id.clone().to_string().as_bytes())
            .unwrap();
    }
}

#[must_use]
pub(crate) fn read_last_ran_job() -> Option<JobId> {
    if let Some(path) = get_cache_dir() {
        let path = path.join("last_ran");
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        Some(contents.into())
    } else {
        None
    }

}

