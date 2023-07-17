use std::{ops::Deref, path::PathBuf};

use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct JobId(String);

impl Deref for JobId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for JobId {
    fn from(s: String) -> Self {
        JobId(s)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Job {
    pub id: JobId,
    pub command: String,
    pub dir: Option<PathBuf>,
    pub uid: u32,
}

impl Job {
    pub fn new(command: String) -> Self {
        Job {
            id: JobId(format!("{:x}", Uuid::new_v4().as_fields().0)),
            dir: None,
            uid: nix::unistd::Uid::current().as_raw(),
            command,
        }
    }

    pub fn with_dir(mut self, dir: PathBuf) -> Self {
        self.dir = Some(dir);
        self
    }
}
