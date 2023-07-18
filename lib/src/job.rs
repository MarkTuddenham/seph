use std::{fmt::Display, ops::Deref, path::PathBuf};

use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobId(String);

impl Deref for JobId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)?;
        Ok(())
    }
}

impl From<String> for JobId {
    fn from(s: String) -> Self {
        JobId(s)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job {
    pub id: JobId,
    pub command: String,
    pub dir: Option<PathBuf>,
    pub uid: u32,
    pub envs: Option<Vec<(String, String)>>,
}

impl Job {
    pub fn new(command: String) -> Self {
        Job {
            id: JobId(format!("{:x}", Uuid::new_v4().as_fields().0)),
            dir: None,
            uid: nix::unistd::Uid::current().as_raw(),
            envs: None,
            command,
        }
    }

    pub fn with_dir(mut self, dir: PathBuf) -> Self {
        self.dir = Some(dir);
        self
    }

    pub fn with_env_all(mut self) -> Self {
        self.envs = Some(std::env::vars().collect());
        self
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}({})", self.id, self.command).as_str())?;
        Ok(())
    }
}
