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

            println!("{}", job.id);
            send_msg(Message::Schedule(job));
        }
        Commands::Output(args) => {
            // TODO: if we follow the last one, do we want to follow the next one to?
            let id = args.id.unwrap_or_else(|| {
                let stream = send_msg(Message::GetLastJob());
                read_all_from_stream(stream).into()
            });

            let mut stream = if args.follow {
                send_msg(Message::Watch(id))
            } else {
                send_msg(Message::Output(id))
            };

            print_from_stream(&mut stream)

        }
    }

    Ok(())
}

fn send_msg(msg: Message) -> UnixStream {
    let mut stream = UnixStream::connect(SOCKET_PATH).unwrap();
    let ser_job: String = msg.into();
    stream.write_all(ser_job.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    stream
}

fn read_all_from_stream(mut stream: UnixStream) -> String {
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    buf
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
