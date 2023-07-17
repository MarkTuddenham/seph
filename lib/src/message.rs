use std::{io::Read, os::unix::net::UnixStream};

use crate::job::{Job, JobId};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    // When adding a new variant don't forget to add it to the From<&UnixStream> impl
    // and to the expects_reply method
    Schedule(Job),
    // List,
    // Status(JobId),
    // Cancel(JobId),
    Output(JobId),
}

impl Message {
    pub fn expects_reply(&self) -> bool {
        match self {
            Message::Schedule(_) => false,
            Message::Output(_) => true,
        }
    }
}

impl From<&mut UnixStream> for Message {
    fn from(stream: &mut UnixStream) -> Self {
        let mut s = String::new();
        stream.read_to_string(&mut s).unwrap();

        let (msg_type, internal) = s.split_once(',').unwrap();

        let mut reader_builder = csv::ReaderBuilder::new();
        reader_builder.has_headers(false);
        let mut reader = reader_builder.from_reader(internal.as_bytes());

        match msg_type {
            "Schedule" => Message::Schedule(reader.deserialize().next().unwrap().unwrap()),
            "Output" => Message::Output(reader.deserialize().next().unwrap().unwrap()),
            _ => panic!("Unknown message type"),
        }
    }
}

impl From<Message> for String {
    fn from(msg: Message) -> Self {
        let mut buf = vec![];
        {
            let mut writer_builder = csv::WriterBuilder::new();
            writer_builder.has_headers(false);
            let mut writer = writer_builder.from_writer(&mut buf);
            writer.serialize(&msg).unwrap();
        }

        let s = std::str::from_utf8(buf.as_slice()).unwrap().to_string();
        s
    }
}
