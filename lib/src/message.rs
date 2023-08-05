use std::{io::Read, os::unix::net::UnixStream};

use crate::job::{Job, JobId};

// TODO: Only allow Messages to be passed over streams, how would this work for an open stream e.g.
// the watch cmd, send a WatchReplyStart and WatchReplyEnd?
// TODO: watch hogs the connection? Maybe it should start its own separate connection?
// TODO: make sure the delimiters are escaped in strings and not strings represented by quatations or we should switch to a better serialisation format

const DELIMITER: char = ':';

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
    Watch(JobId),
    GetLastJob(),
}

impl From<&mut UnixStream> for Message {
    fn from(stream: &mut UnixStream) -> Self {
        let mut s = String::new();
        stream.read_to_string(&mut s).unwrap();
        trim_newline(&mut s);

        tracing::trace!("Deserialising Message from \"{s}\"",);

        let (msg_type, internal) = s.split_once(DELIMITER).unwrap_or((s.as_str(), ""));

        tracing::trace!("Message type: \"{}\"", msg_type);
        tracing::trace!("Message internal: \"{}\"", internal);

        let mut reader_builder = csv::ReaderBuilder::new();
        reader_builder.has_headers(false);
        reader_builder.delimiter(DELIMITER as u8);
        let mut reader = reader_builder.from_reader(internal.as_bytes());

        match msg_type {
            "Schedule" => Message::Schedule(reader.deserialize().next().unwrap().unwrap()),
            "Output" => Message::Output(reader.deserialize().next().unwrap().unwrap()),
            "Watch" => Message::Watch(reader.deserialize().next().unwrap().unwrap()),
            "GetLastJob" => Message::GetLastJob(),
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
            writer_builder.delimiter(DELIMITER as u8);
            let mut writer = writer_builder.from_writer(&mut buf);
            writer.serialize(&msg).unwrap();
        }

        let s = std::str::from_utf8(buf.as_slice()).unwrap().to_string();
        tracing::trace!("Serialised Message to \"{s}\"");
        s
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
