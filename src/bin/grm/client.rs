use gpm::sgcp::*;
use prost::Message;
use std::io::{self, Read, Write};
use std::net::TcpStream;

// TODO: @kumarpit edit this to keep a long lived connection

pub struct GpmClient {}

impl GpmClient {
    pub fn send(component: gpm::sgcp::Resource, task: i32) -> io::Result<()> {
        let mut stream = TcpStream::connect("127.0.0.1:4760")?;
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

        stream.write(&msg.encoded_len().to_be_bytes())?;
        stream.write(&buf)?;
        stream.flush()?;

        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);

        Ok(())
    }
}
