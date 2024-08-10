#![allow(warnings)]

// This file contains the main TCP connection loop and is responsible for
// delegating incoming commands to the appropiate resource mamagers.
mod config;
mod macros;
mod managers;
mod server;
mod connection;
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
use connection::Connection;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::Semaphore;

use crate::managers::Manager;
use crate::managers::ResourceManager;

/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

import_sgcp!();

/// Provides boilerplate to initialize a resource manager and run it in its own (green) thread
macro_rules! init_resource_managers {
    {$($resource:expr => $variant:expr),*} => {{
        let mut map = HashMap::new();
        $(
            info!("Initialising {:?} resource manager task", $resource.as_str_name());
            let mut manager = $variant;
            map.insert($resource.as_str_name().to_string(), manager.tx());
            tokio::spawn(async move { manager.run().await; });
        )*
        map
    }};
}

#[tokio::main]
async fn main() {
    config::init();
    let manager_channel_map = init_resource_managers! {
        Resource::Bms => Manager::<Bms>::new(),
        Resource::Emg => Manager::<Emg>::new(),
        Resource::Maestro => Manager::<Maestro>::new()
    };
    tokio::spawn(telemetry::init());
    server::init(manager_channel_map).await;
}
