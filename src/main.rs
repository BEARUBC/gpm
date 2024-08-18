#![allow(warnings)]

// This file contains the main TCP connection loop and is responsible for
// delegating incoming commands to the appropiate resource managers.
mod config;
mod connection;
mod exporter;
mod macros;
mod managers;
mod server;

use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result;
use bytes::BytesMut;
use config::GPM_TCP_ADDR;
use config::MAX_CONCURRENT_CONNECTIONS;
use connection::Connection;
use exporter::Exporter;
use log::*;
use managers::Bms;
use managers::Emg;
use managers::Maestro;
use managers::ManagerChannelData;
use prost::Message;
#[cfg(feature = "pi")]
use rppal::gpio::Gpio;
#[cfg(feature = "pi")]
use rppal::gpio::InputPin;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::Semaphore;
use tokio::time::sleep;

use crate::managers::Manager;
use crate::managers::ResourceManager;

/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;
const PIN_TO_MONITOR: i32 = 2;

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

#[cfg(feature = "pi")]
async fn start_monitoring_pin(maestro_tx: Sender<ManagerChannelData>) {
    info!("Started GPIO pin monitor for pin {:?}", PIN_TO_MONITOR);
    let gpio = Gpio::new().expect("Failed to initialize GPIO");
    let mut pin = gpio
        .get(PIN_TO_MONITOR)
        .expect("Failed to access pin")
        .into_input_pullup();
    loop {
        let resp_tx = oneshot::channel::<String>();
        if pin.is_high() {
            maestro_tx.send(ManagerChannelData {
                task_code: sgcp::maestro::Task::OpenFist.as_str_name().to_string(),
                task_data: None,
                resp_tx,
            });
        } else {
            maestro_tx.send(ManagerChannelData {
                task_code: sgcp::maestro::Task::CloseFist.as_str_name().to_string(),
                task_data: None,
                resp_tx,
            });
        }
        let res = resp_rx.await.unwrap();
        info!("Receieved response from Maestro manager: {:?}", res);
        sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() {
    config::init();
    let manager_channel_map = init_resource_managers! {
        Resource::Bms => Manager::<Bms>::new(),
        Resource::Emg => Manager::<Emg>::new(),
        Resource::Maestro => Manager::<Maestro>::new()
    };
    tokio::spawn(async {
        let mut exporter = Exporter::new();
        exporter.init().await
    });

    // #[cfg(feature = "pi")]
    // {
    //     let maestro_tx = manager_channel_map
    //         .get(Resource::Maestro.as_str_name())
    //         .clone();
    //     tokio::spawn(async move {
    //         start_monitoring_pin(maestro_tx).await;
    //     });
    // }
    
    server::init(manager_channel_map).await;
}
