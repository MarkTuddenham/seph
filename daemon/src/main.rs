mod handlers;
mod utils;
mod worker;

use crate::{utils::get_cache_dir, worker::Worker};

//TODO: save the jobs & ouputs in a mysql database

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

    let res = Worker::new().run();

    if let Err(e) = res {
        tracing::error!("Error: {}", e);
    }
}
