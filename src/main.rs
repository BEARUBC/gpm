// This file contains the main TCPListener loop and is responsible for
// dispatching the required task.
mod taskdefs;

use anyhow::Result;
use tokio::{
    io::AsyncReadExt, 
    net::{
        TcpListener, 
        TcpStream
    }
};
use taskdefs::*;
use prost::Message;
use std::io::Cursor;
use bytes::BytesMut;

pub mod sgcp {
    include!(concat!(env!("OUT_DIR"), "/sgcp.rs"));
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4760").await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(stream).await.unwrap();
        });
    }
}

// Parses protobuf struct from stream and handles the request.
async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    match stream.read_buf(&mut buf).await {
        Ok(0) => {
            println!("Could not read incoming request, connection closed.");
        },
        Ok(_) => {
            let req = deserialize_sgcp_request(&mut buf).unwrap();
            handle_task(req).unwrap();
        }
        Err(e) => {
            println!("Failed to read from socket; err = {:?}", e);
        }
    }
    Ok(())
}

// @todo: look into creating a macro to reduce duplication. also because macros are cool.
fn handle_task(request: sgcp::CommandRequest) -> Result<()> {
    match request.component() {
        sgcp::Component::Bms => {
            println!("Dispatching BMS task");
            tokio::spawn(bms::check_battery_usage());
        }
        sgcp::Component::Emg => {
            println!("Dispatching EMG task");
            tokio::spawn(emg::read_edc());
        }
        sgcp::Component::Servo => {
            println!("Dispatching SERVO task");
            // tokio::spawn(servo::handle_servo_task(request.task_code));
        }
        sgcp::Component::Telemetry => {
            println!("Dispatching TELEMETRY task");
            tokio::spawn(telemetry::handle_telemetry_task(request.task_code));
        }
        _ => {
            println!("Unmatched task, ignoring...")
        }
    }
    Ok(())
}

pub fn deserialize_sgcp_request(buf: &mut BytesMut) -> Result<sgcp::CommandRequest, prost::DecodeError> {
    sgcp::CommandRequest::decode(&mut Cursor::new(buf))
}
