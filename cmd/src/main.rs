//! Command line tool to schedule jobs on a single workstation

mod args;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use libseph::{Job, Message, SOCKET_PATH};

use crate::args::{parse_args, Commands};

fn main() -> anyhow::Result<()> {
    let args = parse_args();

    match args.command {
        Commands::Run(run) => {
            let mut job = Job::new(run.command);

            if !run.ignore_run_dir {
                job = job.with_dir(std::env::current_dir()?);
            }

            // TOOD?: Capture env
            send_msg(Message::Schedule(job));
        } 
        Commands::Output(job_id) => {
            send_msg(Message::Output(job_id.id.into()));
        }

        // args::Commands::List => {
          //     println!("List");
          // }
          // args::Commands::Kill(kill) => {
          //     println!("Kill: {:?}", kill);
          // }
    }

    Ok(())
}

fn send_msg(msg: Message) {
    if let Message::Schedule(ref job) = msg {
        println!("{}", *job.id)
    }

    let mut stream = UnixStream::connect(SOCKET_PATH).unwrap();
    let ser_job: String = msg.clone().into();
    stream.write_all(ser_job.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    match msg {
        Message::Output(_) => {
            print_from_stream(&mut stream);
        }
        _ => {}
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
