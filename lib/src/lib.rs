mod job;
mod message;

pub use job::{Job, JobId};
pub use message::Message;

pub const SOCKET_PATH: &str = "/run/seph.socket";



