#![allow(warnings)]

// This file contains the main TCP connection loop and is responsible for
// delegating incoming commands to the appropiate resource mamagers.
mod config;
mod macros;
mod managers;
mod server;
mod streaming;
mod telemetry;

use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result;
use bytes::BytesMut;
use config::GPM_TCP_ADDR;
use config::MAX_TCP_CONNECTIONS;
use log::*;
use managers::ManagerChannelData;
use prost::Message;
use streaming::Connection;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::Semaphore;

use crate::_dispatch_task as dispatch_task;
use crate::_init_resource_managers as init_resource_managers;
use crate::managers::Bms;
use crate::managers::Emg;
use crate::managers::Maestro;
use crate::managers::Manager;
use crate::managers::ResourceManager;

type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

// Import protobuf generated code to handle de/serialization
import_sgcp!();

#[tokio::main]
async fn main() {
    config::init();
    tokio::spawn(telemetry::http::start_server());
    server::init_gpm_listener(init_resource_managers().await).await;
}

// Initializes the resource managers and returns a map containing the mpsc
// channels to each manager
async fn init_resource_managers() -> ManagerChannelMap {
    init_resource_managers! {
        Component::Bms => Manager::BmsManager(Bms::new()),
        Component::Emg => Manager::EmgManager(Emg::new()),
        Component::Maestro => Manager::MaestroManager(Maestro::new())
    }
}
