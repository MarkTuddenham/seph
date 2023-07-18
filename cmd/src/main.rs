//! Command line tool to schedule jobs on a single workstation

mod args;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use libseph::{Job, Message, SOCKET_PATH};

use crate::args::{parse_args, Commands};

fn main() -> anyhow::Result<()> {
    let args = parse_args();

    match args.command {
        Commands::Run(run_args) => {
            let mut job = Job::new(run_args.command);

            if !run_args.ignore_run_dir {
                job = job.with_dir(std::env::current_dir()?);
            }

            if run_args.env_capture_all {
                job = job.with_env_all();
            }

            // TODO?: Capture env
            send_msg(Message::Schedule(job));
        }
        Commands::Output(job_id) => {
            send_msg(Message::Output(job_id.id));
        }
        Commands::Watch(job_id) => {
            send_msg(Message::Watch(job_id.id));
        }
    }

    Ok(())
}

fn send_msg(msg: Message) {
    if let Message::Schedule(ref job) = msg {
        println!("{}", job.id)
    }

    let mut stream = UnixStream::connect(SOCKET_PATH).unwrap();
    let ser_job: String = msg.clone().into();
    stream.write_all(ser_job.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    if msg.expects_reply() {
        print_from_stream(&mut stream);
    }
}

fn print_from_stream(stream: &mut UnixStream) {
    let mut buf = [0; 1024];
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        print!("{}", String::from_utf8_lossy(&buf[..n]));
    }
}
