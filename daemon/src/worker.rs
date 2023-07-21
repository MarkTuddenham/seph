use std::collections::VecDeque;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use libseph::{Job, SOCKET_PATH};

use crate::{handlers::handle_client, utils::{get_cache_dir, write_last_ran_job}};

pub(crate) struct Worker {
    process_mutex: Mutex<()>,
    jobs: Mutex<VecDeque<Job>>,
}

impl Worker {
    pub(crate) fn new() -> Self {
        Self {
            process_mutex: Mutex::new(()),
            jobs: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn run(self) -> anyhow::Result<()> {
        // Remove the socket if it already exists
        let listener = bind_socket()?;
        let arc_self = Arc::new(self);

        loop {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let arc_worker = arc_self.clone();
                        spawn(|| handle_client(arc_worker, stream))
                    }
                    Err(_) => {
                        break;
                    }
                };
            }
        }
    }

    pub(crate) fn add_job(&self, job: Job) {
        tracing::debug!("Added job to queue");
        let mut jobs = self.jobs.lock().unwrap();
        jobs.push_back(job);
    }

    pub(crate) fn process_jobs(&self) {
        tracing::debug!("Trying to get process lock");
        // If we can't get the lock, it means we're already processing a job
        let process_lock = self.process_mutex.try_lock();

        if process_lock.is_err() {
            tracing::debug!("Already processing jobs");
            return;
        }

        tracing::debug!("Starting to processing jobs");

        loop {
            let job = {
                let mut jobs = self.jobs.lock().unwrap();
                jobs.pop_front()
            };

            if let Some(job) = job {
                write_last_ran_job(&job);
                exec_job(job);
            } else {
                tracing::debug!("No more jobs to process.");
                break;
            }
        }
    }
}

fn exec_job(job: Job) {
    tracing::info!("Starting job: {job}");
    let mut command = Command::new("sh");

    let (stdout, stderr) = get_stdios(&job);

    command
        .arg("-c")
        .arg(&job.command)
        .env_clear()
        .uid(job.uid)
        .gid(job.uid)
        .stdout(stdout)
        .stderr(stderr);

    tracing::trace!("Running as uid: {}", job.uid);

    if let Some(envs) = job.envs.clone() {
        tracing::trace!("Setting envs: {envs:?}", envs = envs);
        command.envs(envs);
    }

    if let Some(ref dir) = job.dir {
        tracing::trace!("Running in dir: {dir:?}");
        command.current_dir(dir);
    }

    let status = command.status().unwrap();
    tracing::info!("Finished job: {}; {}", status, job);
}

fn get_stdios(job: &Job) -> (Stdio, Stdio) {
    if let Some(path) = get_cache_dir() {
        let path = path.join(job.id.clone().to_string());
        let file = File::create(path).unwrap();
        let err_file = file.try_clone().unwrap();
        (Stdio::from(file), Stdio::from(err_file))
    } else {
        (Stdio::null(), Stdio::null())
    }
}

fn bind_socket() -> anyhow::Result<UnixListener> {
    let rm_result = std::fs::remove_file(SOCKET_PATH);
    if let Err(e) = rm_result {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::error!("Error removing previous socket: {}", e);
            return Err(e.into());
        }
    }

    let listener = UnixListener::bind(SOCKET_PATH)?;
    tracing::info!("Listening at {}", SOCKET_PATH);

    let mut perms = std::fs::metadata(SOCKET_PATH)?.permissions();
    perms.set_mode(0o777);
    std::fs::set_permissions(SOCKET_PATH, perms)?;

    Ok(listener)
}
