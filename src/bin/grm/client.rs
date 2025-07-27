//! GPM TCP Client

use gpm::sgcp::*;
use prost::Message;
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub struct GpmClient {
    stream: TcpStream,
}

pub struct GpmResponse {
    pub message: String,
    pub resource: Resource,
    pub task_code: String,
}

impl GpmClient {
    pub fn new() -> Self {
        Self {
            stream: TcpStream::connect("127.0.0.1:4760").unwrap(),
        }
    }

    /// Sends command request to GPM
    pub fn send(&mut self, component: gpm::sgcp::Resource, task: i32) -> io::Result<GpmResponse> {
        let mut msg = gpm::sgcp::Request::default();

        msg.resource = component as i32;
        msg.task_code = match component {
            Resource::UndefinedComponent => "UNDEFINED".to_string(),
            Resource::Bms => bms::Task::try_from(task).unwrap().as_str_name().to_string(),
            Resource::Emg => emg::Task::try_from(task).unwrap().as_str_name().to_string(),
            Resource::Maestro => maestro::Task::try_from(task)
                .unwrap()
                .as_str_name()
                .to_string(),
        };

        let mut buf = Vec::new();
        buf.reserve(msg.encoded_len());
        msg.encode(&mut buf).unwrap();

        self.stream.write(&msg.encoded_len().to_be_bytes())?;
        self.stream.write(&buf)?;
        self.stream.flush()?;

        let mut buffer = [0; 512];
        let bytes_read = self.stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);

        Ok(GpmResponse {
            message: response.into_owned(),
            resource: msg.resource(),
            task_code: msg.task_code,
        })
    }
}
