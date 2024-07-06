// This file contains the main TCPListener loop and is responsible for
// dispatching the required task.
mod task_definitions;
mod telemetry;

use anyhow::Result;
use tokio::{
    io::AsyncReadExt, 
    net::{
        TcpListener, 
        TcpStream
    }
};
// use taskdefs::*;
use prost::Message;
use std::io::Cursor;
use bytes::BytesMut;

pub mod sgcp {
    include!(concat!(env!("OUT_DIR"), "/sgcp.rs"));
}

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    println!(r"  ________                                
 /  _____/_______ _____     ____________  
/   \  ___\_  __ \\__  \   /  ___/\____ \ 
\    \_\  \|  | \/ / __ \_ \___ \ |  |_| |
 \______  /|__|   (____  //____  ||   __/ 
        \/             \/      \/ |__|    ");
    println!("Developed at UBC Bionics | Version 1.0.0");
    let _ = telemetry::http::start_server().await;
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
            error!("Could not read incoming request, connection closed.");
        },
        Ok(_) => {
            let req = deserialize_sgcp_request(&mut buf).unwrap();
            handle_task(req).unwrap();
        }
        Err(e) => {
            error!("Failed to read from socket; err = {:?}", e);
        }
    }
    Ok(())
}

// @todo: look into creating a macro to reduce duplication. also because macros are cool.
fn handle_task(request: sgcp::CommandRequest) -> Result<()> {
    match request.component() {
        sgcp::Component::Emg => {
            info!("Dispatching EMG task");
            // tokio::spawn(emg::read_edc());
        }
        sgcp::Component::Servo => {
            info!("Dispatching SERVO task");
            // tokio::spawn(servo::handle_servo_task(request.task_code));
        }
        _ => {
            info!("Unmatched task, ignoring...")
        }
    }
    Ok(())
}

pub fn deserialize_sgcp_request(buf: &mut BytesMut) -> Result<sgcp::CommandRequest, prost::DecodeError> {
    sgcp::CommandRequest::decode(&mut Cursor::new(buf))
}
