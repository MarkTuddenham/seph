use notify::{Config, PollWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;

use libseph::{Job, JobId, Message};

use crate::{utils::get_cache_dir, worker::Worker};

pub(crate) fn handle_client(worker: Arc<Worker>, mut stream: UnixStream) {
    tracing::debug!("Client connected");
    let msg = Message::from(&mut stream);
    tracing::info!("Received message: {:?}", msg);

    match msg {
        Message::Schedule(job) => handle_schedule(worker, job),
        Message::Output(job_id) => handle_get_output(&mut stream, job_id),
        Message::Watch(job_id) => handle_watch_output(&mut stream, job_id),
    }
}

fn handle_schedule(worker: Arc<Worker>, job: Job) {
    tracing::debug!("Scheduling job: {}", job);
    worker.add_job(job);
    worker.process_jobs();
}

fn handle_get_output(stream: &mut UnixStream, job_id: JobId) {
    tracing::debug!("Getting output for job: {job_id}");
    let path = get_cache_dir().unwrap().join(job_id.to_string());
    let file = File::open(path).unwrap();
    let res = send_output(stream, file, 0, [0; 1024].as_mut());
    if res.is_err() {
        tracing::warn!("Client disconnected before all output was sent");
    }
}

fn handle_watch_output(stream: &mut UnixStream, job_id: JobId) {
    tracing::debug!("Watching output for job: {job_id}");

    let path = get_cache_dir().unwrap().join(job_id.to_string());

    let config = Config::default()
        .with_compare_contents(true) // crucial part for pseudo filesystems
        .with_poll_interval(Duration::from_secs(2));

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = PollWatcher::new(tx, config).unwrap();
    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    let mut buf = [0; 1024];
    let mut cursor = {
        let file = File::open(&path).unwrap();
        let cursor = send_output(stream, file, 0, &mut buf);
        if cursor.is_err() {
            tracing::warn!("Client disconnected");
            return;
        }

        cursor.unwrap()
    };

    for res in rx {
        match res {
            Ok(_event) => {
                tracing::debug!("File changed event, sending output for job: {job_id}");
                let file = File::open(&path).unwrap();
                cursor = match send_output(stream, file, cursor, &mut buf) {
                    Ok(cursor) => cursor,
                    Err(_) => {
                        tracing::warn!("Client disconnected while sending output");
                        return;
                    }
                }
            }
            Err(e) => tracing::error!("Watch error: {:?}", e),
        }
    }
}

fn send_output(
    stream: &mut UnixStream,
    mut file: File,
    mut cursor: u64,
    buf: &mut [u8],
) -> Result<u64, std::io::Error> {
    file.seek(std::io::SeekFrom::Start(cursor)).unwrap();
    loop {
        let bytes_read = file.read(buf).unwrap();
        if bytes_read == 0 {
            return Ok(cursor);
        }
        cursor += bytes_read as u64;
        stream.write_all(&buf[..bytes_read])?;
    }
}
