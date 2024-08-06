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
use config::MAX_CONCURRENT_CONNECTIONS;
use log::*;
use managers::Bms;
use managers::Emg;
use managers::Maestro;
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
use crate::managers::Manager;
use crate::managers::ResourceManager;

/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

import_sgcp!();

#[tokio::main]
async fn main() {
    config::init();
    tokio::spawn(telemetry::http::start_server());
    let manager_channel_map = init_resource_managers().await;
    server::init_gpm_listener(manager_channel_map).await;
}

/// Initializes the resource managers and returns a map containing the mpsc
/// channels to each manager
async fn init_resource_managers() -> ManagerChannelMap {
    init_resource_managers! {
        Resource::Bms => Manager::<Bms>::new(),
        Resource::Emg => Manager::<Emg>::new(),
        Resource::Maestro => Manager::<Maestro>::new()
    }
}
