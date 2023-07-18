//! Command line arguments

use clap::{Parser, Subcommand};
use libseph::JobId;

// /// When to abort a job with multiple runs
// #[derive(clap::ValueEnum, Clone, Debug)]
// pub(crate) enum AbortVaraint {
//     Never,
//     IfFirst,
//     Always,
// }

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
pub(crate) enum Commands {
    Run(RunCommand),
    Output(OutputCommand),
    Watch(WatchCommand),
    // List,
    // Cancel(CancelCommand),
    // Reorder(ReorderCommand),
}

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct RunCommand {
    // #[clap(short, long, default_value = "1")]
    // pub(crate) count: u32,
    //
    // #[clap(short, long, value_enum, default_value = "never")]
    // pub(crate) abort: AbortVaraint,

    #[clap(short, long)]
    pub(crate) ignore_run_dir: bool,

    #[clap(short, long)]
    pub(crate) env_capture_all: bool,

    pub(crate) command: String,
}

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct OutputCommand {
    pub(crate) id: JobId,
}

#[derive(clap::Args, Clone, Debug)]
pub(crate) struct WatchCommand {
    pub(crate) id: JobId,
}


pub(crate) fn parse_args() -> Args {
    Args::parse()
}
