#![allow(warnings)] // ONLY FOR CLEANER LOGS DURING DEBUGGING

// This file contains the main TCP connection loop and is responsible for
// delegating incoming commands to the appropiate resource mamagers.
mod managers;
mod telemetry;
mod config;

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, 
    net::{
        TcpListener, 
        TcpStream
    },
    sync::{mpsc::Sender, oneshot}
};
use prost::Message;
use std::io::Cursor;
use bytes::BytesMut;
use std::collections::HashMap;

use crate::managers::ResourceManager;

// Import protobuf generated code to handle de/serialization
pub mod sgcp {
    include!(concat!(env!("OUT_DIR"), "/sgcp.rs"));
    pub mod bms {
        include!(concat!(env!("OUT_DIR"), "/sgcp.bms.rs"));
    }
    pub mod emg {
        include!(concat!(env!("OUT_DIR"), "/sgcp.emg.rs"));
    }
    pub mod maestro {
        include!(concat!(env!("OUT_DIR"), "/sgcp.maestro.rs"));
    }
}

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[tokio::main]
async fn main() {
    config::init();
    let send_channel_map = init_resources().await;
    let listener = TcpListener::bind("127.0.0.1:4760").await.unwrap();
    loop {
        // @todo: krarpit need to bound the number of active connections GPM can maintain
        let (stream, _) = listener.accept().await.unwrap();
        let send_channel_map = send_channel_map.clone();
        tokio::spawn(async move {
            handle_connection(stream, &send_channel_map).await.unwrap();
        });
    }
}

async fn init_resources() -> HashMap<String, Sender<(i32, oneshot::Sender<std::string::String>)>> {
    // let _ = telemetry::http::start_server().await;
    let mut map = HashMap::new();
    {
        let mut bms_manager = managers::bms::BMS::new();
        bms_manager.init().unwrap();
        map.insert(sgcp::Component::Bms.as_str_name().to_string(), bms_manager.tx());
        tokio::spawn(async move {
            bms_manager.run().await;
        });
    }
    {
        let mut emg_manager = managers::emg::EMG::new();
        emg_manager.init().unwrap();
        map.insert(sgcp::Component::Emg.as_str_name().to_string(), emg_manager.tx());
        tokio::spawn(async move {
            emg_manager.run().await;
        });
    }
    {
        let mut maestro_manager = managers::maestro::Maestro::new();
        maestro_manager.init().unwrap();
        map.insert(sgcp::Component::Maestro.as_str_name().to_string(), maestro_manager.tx());
        tokio::spawn(async move {
            maestro_manager.run().await;
        });
    }
    map
}

// Parses protobuf struct from stream and handles the request.
async fn handle_connection(mut stream: TcpStream, map: &HashMap<String, Sender<(i32, oneshot::Sender<std::string::String>)>>) -> Result<()> {
    // @todo: krarpit implement framing abstraction for tcp stream
    let mut buf = BytesMut::with_capacity(1024);
    match stream.read_buf(&mut buf).await {
        Ok(0) => {
            error!("Could not read incoming request, connection closed.");
        },
        Ok(_) => {
            let req = deserialize_sgcp_request(&mut buf).unwrap();
            let res = handle_task(req, &map).await.unwrap();
            stream.write(res.as_bytes()).await.unwrap();
        }
        Err(e) => {
            error!("Failed to read from socket; err={:?}", e);
        }
    }
    Ok(())
}

// @todo krarpit look into creating a macro to reduce duplication
async fn handle_task(request: sgcp::Request, map: &HashMap<String, Sender<(i32, oneshot::Sender<std::string::String>)>>) -> Result<String> {
    match request.component() {
        sgcp::Component::Emg => {
            info!("Dispatching EMG task with task_code={:?}", request.task_code);
            match map.get("EMG") {
                Some(tx) => {
                    let (resp_tx, resp_rx) = oneshot::channel::<String>();
                    tx.send((request.task_code, resp_tx)).await.unwrap();
                    let res = resp_rx.await;
                    info!("EMG task returned value={:?}", res);
                    return Ok(res?);
                },
                None => error!("EMG resource manager not initialized")
            }
        }
        sgcp::Component::Maestro => {
            info!("Dispatching MAESTRO task");
            match map.get("MAESTRO") {
                Some(tx) => {
                    let (resp_tx, resp_rx) = oneshot::channel::<String>();
                    tx.send((request.task_code, resp_tx)).await.unwrap();
                    let res = resp_rx.await;
                    info!("Maestro task returned value={:?}", res);
                    return Ok(res?);
                },
                None => error!("Maestro resource manager not initialized")
            }
        }
        sgcp::Component::Bms => {
            info!("Dispatching BMS task");
            match map.get("BMS") {
                Some(tx) => {
                    let (resp_tx, resp_rx) = oneshot::channel::<String>();
                    tx.send((request.task_code, resp_tx)).await.unwrap();
                    let res = resp_rx.await;
                    info!("BMS task returned value={:?}", res);
                    return Ok(res?);
                },
                None => error!("BMS resource manager not initialized")
            }
        }
        _ => {
            info!("Unmatched task, ignoring...");
        }
    }
    Ok("Undefined component".to_string())
}

pub fn deserialize_sgcp_request(buf: &mut BytesMut) -> Result<sgcp::Request, prost::DecodeError> {
    sgcp::Request::decode(&mut Cursor::new(buf))
}
