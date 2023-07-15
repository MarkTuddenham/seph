use std::collections::VecDeque;

use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use directories::ProjectDirs;

use libseph::{Job, Message, SOCKET_PATH, JobId};

struct Worker {
    process_mutex: Mutex<()>,
    jobs: Mutex<VecDeque<Job>>,
}

impl Worker {
    fn run(self) -> anyhow::Result<()> {
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

fn handle_client(worker: Arc<Worker>, mut stream:UnixStream) {
    tracing::debug!("Client connected");
    let msg = Message::from(&mut stream);
    tracing::info!("Received msg: {:?}", msg);

    match msg {
        Message::Schedule(job) => handle_schedule(worker, job),
        Message::Output(job_id) => handle_get_output(&mut stream, job_id),
    }

}

fn handle_schedule(worker: Arc<Worker>, job: Job) {
    {
        let mut jobs = worker.jobs.lock().unwrap();
        jobs.push_back(job);
        tracing::debug!("Added job to queue");
    }

    process_jobs(worker);
}

fn handle_get_output(stream: &mut UnixStream, job_id: JobId) {
    tracing::debug!("Getting output for job: {:?}", job_id);
    let path = get_cache_dir().unwrap().join(job_id.to_string());
    let mut file = File::open(path).unwrap();
    let mut buf = [0; 1024];
    loop {
        let bytes_read = file.read(&mut buf).unwrap();
        if bytes_read == 0 {
            break;
        }

        let res = stream.write_all(&buf[..bytes_read]);

        // stop sending if the client disconnects
        if res.is_err() {
            tracing::warn!("Client disconnected");
            break;
        }
    }
}



fn process_jobs(worker: Arc<Worker>) {
    tracing::debug!("Trying to get process lock");
    // If we can't get the lock, it means we're already processing a job
    let process_lock = worker.process_mutex.try_lock();

    if process_lock.is_err() {
        tracing::debug!("Already processing jobs");
        return;
    }

    tracing::debug!("Starting to processing jobs");

    loop {
        // let _process_lock = process_lock.unwrap();

        let job = {
            let mut jobs = worker.jobs.lock().unwrap();
            jobs.pop_front()
        };

        if let Some(job) = job {
            exec_job(job);
        } else {
            tracing::debug!("No more jobs to process.");
            break;
        }
    }
}

fn exec_job(job: Job) {
    tracing::info!("Starting job: {:?}", job);
    let mut command = Command::new("sh");


    command
        .arg("-c")
        .arg(&job.command)
        .env_clear()
        .stdout(get_stdio(&job))
        .stderr(get_stdio(&job));

    if let Some(ref dir) = job.dir {
        command.current_dir(dir);
    }

    let status = command.status().unwrap();
    tracing::info!("Finished job: {}; {:?}", status, job);
}


fn get_stdio(job: &Job) -> Stdio {
    if let Some(path) = get_cache_dir() {
        let path = path.join(job.id.clone().to_string());
        let file = File::create(path).unwrap();
        Stdio::from(file)
    } else {
        Stdio::null()
    }
}

fn get_cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("dev", "tudders", "seph").map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

fn main() {
    let cache_dir = get_cache_dir();
    if let Some(log_dir) = &cache_dir {
        use tracing_subscriber::prelude::*;

        let file_appender = tracing_appender::rolling::daily(log_dir, "seph.log");

        let file_log = tracing_subscriber::fmt::Layer::new()
            .with_writer(file_appender)
            .with_ansi(false);

        let std_log = tracing_subscriber::fmt::Layer::new()
            .with_writer(std::io::stdout.with_max_level(tracing::Level::INFO))
            .with_ansi(true);

        let _res = tracing_subscriber::registry()
            .with(file_log)
            .with(std_log)
            .try_init();

        tracing::info!("Logging to: {:?}/", log_dir);
    }

    let res = Worker {
        process_mutex: Mutex::new(()),
        jobs: Mutex::new(VecDeque::new()),
    }
    .run();

    if let Err(e) = res {
        tracing::error!("Error: {}", e);
    }
}
