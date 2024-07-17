#![allow(warnings)]

// This file contains the main TCP connection loop and is responsible for
// delegating incoming commands to the appropiate resource mamagers.
mod config;
mod macros;
mod managers;
mod telemetry;

use config::{GPM_TCP_ADDR, MAX_TCP_CONNECTIONS};
use log::*;
use anyhow::Result;
use bytes::BytesMut;
use managers::ManagerChannelData;
use prost::Message;
use std::collections::HashMap;
use std::sync::Arc;
use std::{any::Any, io::Cursor};
use tokio::sync::Semaphore;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Sender, oneshot},
};
use crate::_dispatch_task as dispatch_task;
use crate::_init_resource_managers as init_resource_managers;
use crate::managers::{Bms, Emg, Maestro, Manager, ResourceManager};

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
    // boot up
    config::init();
    let manager_channel_map = init_resource_managers().await;
    // tokio::spawn(telemetry::http::start_server());

    let listener = TcpListener::bind(GPM_TCP_ADDR).await.unwrap();
    let sem = Arc::new(Semaphore::new(MAX_TCP_CONNECTIONS));
    info!("Listening on {:?}", GPM_TCP_ADDR);
    loop {
        let sem_clone = Arc::clone(&sem);
        let (stream, client_addr) = listener.accept().await.unwrap();
        let send_channel_map = manager_channel_map.clone();
        tokio::spawn(async move {
            let aq = sem_clone.try_acquire();
            if let Ok(_) = aq {
                info!("Accpeted new remote connection from host={:?}", client_addr);
                handle_connection(stream, &send_channel_map).await.unwrap();
            } else {
                error!("Rejected new remote connection from host={:?}, currently serving maximum_clients={:?}", client_addr, MAX_TCP_CONNECTIONS)
            }
        });
    }
}

// Parses protobuf struct from stream and handles the request.
async fn handle_connection(mut stream: TcpStream, map: &ManagerChannelMap) -> Result<()> {
    // @todo: krarpit implement framing abstraction for tcp stream
    let mut buf = BytesMut::with_capacity(1024);
    match stream.read_buf(&mut buf).await {
        Ok(0) => {
            error!("Could not read incoming request, connection closed.");
        }
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

pub fn deserialize_sgcp_request(buf: &mut BytesMut) -> Result<sgcp::Request, prost::DecodeError> {
    sgcp::Request::decode(&mut Cursor::new(buf))
}

init_resource_managers! {
    Component::Bms => Manager::BmsManager(Bms::new()),
    Component::Emg => Manager::EmgManager(Emg::new()),
    Component::Maestro => Manager::MaestroManager(Maestro::new())
}

dispatch_task! {
    Component::Bms => (bms::Task, request::TaskData::BmsData, ManagerChannelData::BmsChannelData),
    Component::Emg => (emg::Task, request::TaskData::EmgData, ManagerChannelData::EmgChannelData),
    Component::Maestro => (maestro::Task, request::TaskData::MaestroData, ManagerChannelData::MaestroChannelData)
}
