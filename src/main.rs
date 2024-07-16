#![allow(warnings)] // ONLY FOR CLEANER LOGS DURING DEBUGGING

// This file contains the main TCP connection loop and is responsible for
// delegating incoming commands to the appropiate resource mamagers.
mod managers;
mod telemetry;
mod config;
mod macros;

use log::*;

use anyhow::Result;
use managers::ManagerChannelData;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt}, 
    net::{
        TcpListener, 
        TcpStream
    },
    sync::{mpsc::Sender, oneshot}
};
use prost::Message;
use std::{any::Any, io::Cursor};
use bytes::BytesMut;
use std::collections::HashMap;

use crate::_dispatch_task as dispatch_task;

use crate::managers::{Manager, ResourceManager, Bms, Emg, Maestro};

type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

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

use sgcp::*;

#[tokio::main]
async fn main() {
    config::init();
    let manager_channel_map = init_resource_managers().await;
    tokio::spawn(telemetry::http::start_server());
    let listener = TcpListener::bind("127.0.0.1:4760").await.unwrap();
    info!("Listening on port 4760");
    loop {
        // @todo: krarpit need to bound the number of active connections GPM can maintain
        let (stream, _) = listener.accept().await.unwrap();
        let send_channel_map = manager_channel_map.clone();
        tokio::spawn(async move {
            handle_connection(stream, &send_channel_map).await.unwrap();
        });
    }
}

async fn init_resource_managers() -> ManagerChannelMap {
    let mut map = HashMap::new();
    init_resource_manager(Manager::BmsManager(Bms::new()), Component::Bms, &mut map).await;
    init_resource_manager(Manager::EmgManager(Emg::new()), Component::Emg, &mut map).await;
    init_resource_manager(Manager::MaestroManager(Maestro::new()), Component::Maestro, &mut map).await;
    map
}

async fn init_resource_manager(mut manager: managers::Manager, component: Component, map: &mut ManagerChannelMap) {
    info!("Initializing resource_manager_task={:?}", component.as_str_name());
    manager.init().unwrap();
    map.insert(component.as_str_name().to_string(), manager.tx());
    tokio::spawn(async move { manager.run().await; });
}

// Parses protobuf struct from stream and handles the request.
async fn handle_connection(mut stream: TcpStream, map: &ManagerChannelMap) -> Result<()> {
    // @todo: krarpit implement framing abstraction for tcp stream
    let mut buf = BytesMut::with_capacity(1024);
    match stream.read_buf(&mut buf).await {
        Ok(0) => {
            error!("Could not read incoming request, connection closed.");
        },
        Ok(_) => {
            let req = deserialize_sgcp_request(&mut buf).unwrap();
            let res = dispatch_task(req, &map).await.unwrap();
            stream.write(res.as_bytes()).await.unwrap();
        }
        Err(e) => {
            error!("Failed to read from socket; err={:?}", e);
        }
    }
    Ok(())
}

dispatch_task! {
    Component::Bms => (bms::Task, request::TaskData::BmsData, ManagerChannelData::BmsChannelData),
    Component::Emg => (emg::Task, request::TaskData::EmgData, ManagerChannelData::EmgChannelData),
    Component::Maestro => (maestro::Task, request::TaskData::MaestroData, ManagerChannelData::MaestroChannelData)
}

pub fn deserialize_sgcp_request(buf: &mut BytesMut) -> Result<sgcp::Request, prost::DecodeError> {
    sgcp::Request::decode(&mut Cursor::new(buf))
}
