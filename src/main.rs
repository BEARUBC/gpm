#![allow(warnings)]

// This file contains the main TCP connection loop and is responsible for
// delegating incoming commands to the appropiate resource mamagers.
mod config;
mod streaming;
mod macros;
mod managers;
mod telemetry;
mod server;

use config::{GPM_TCP_ADDR, MAX_TCP_CONNECTIONS};
use streaming::Connection;
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
import_sgcp!();

#[tokio::main]
async fn main() {
    config::init();
    tokio::spawn(telemetry::http::start_server());
    server::init_gpm_listener(init_resource_managers().await).await;
}

// Initializes the resource managers and returns a map containing the mpsc
// channels to each manager
init_resource_managers! {
    Component::Bms => Manager::BmsManager(Bms::new()),
    Component::Emg => Manager::EmgManager(Emg::new()),
    Component::Maestro => Manager::MaestroManager(Maestro::new())
}