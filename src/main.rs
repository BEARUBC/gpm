#![allow(warnings)]

// This file contains the main TCP connection loop
// It is responsible for handling incoming TCP connections, delegating tasks to resource managers, and initializing key components.
mod config; // Configuration settings (e.g., TCP address, buffer sizes)
mod connection; // Handles TCP connection framing and data transmission
mod exporter; // Telemetry exporter for system metrics
mod macros; // Utility macros for common functionality
mod managers; // Resource management framework
mod server; // Main server loop and task routing

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

// GPIO pin to monitor muscle activity (only for Raspberry Pi builds) #[cfg(feature = "pi")]
const PIN_TO_MONITOR: i32 = 2;

// Import protobuf definitions for task communication
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

/// Starts monitoring GPIO pins for muscle activity and triggers appropriate tasks.
/// This is for when the code for the EMG wasn't working and we wanted experiment while using a button on the maestro
/// Change this code to not take data from analytics module but from emg manager
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

async fn receive_EMG(emg_tx: Sender<ManagerChannelData>){
    info!("Started EMG pin monitor for pin {:?}", PIN_TO_MONITOR);
    let gpio = Gpio::new().expect("Failed to initialize GPIO");
    let mut pin = gpio
        .get(PIN_TO_MONITOR)
        .expect("Failed to access pin")
        .into_input_pullup();
    
}

/// Main entry point for the bionic arm system.
/// Initializes all resource managers, telemetry, and starts the TCP server.
#[tokio::main]
async fn main() {
    // Initialize the logger for detailed runtime diagnostics.
    #[cfg(feature = "dev")]
    console_subscriber::init();

    // Load configuration settings (e.g., logging level, server addresses).
    config::init();

    // Initialize resource managers and their communication channels.
    let manager_channel_map = init_resource_managers! {
        Resource::Bms => Manager::<Bms>::new(),
        Resource::Emg => Manager::<Emg>::new(),
        Resource::Maestro => Manager::<Maestro>::new()
    };

    // Spawn the telemetry exporter as an independent async task.
    tokio::spawn(async {
        let mut exporter = Exporter::new();
        exporter.init().await
    });

    // If running on Raspberry Pi, start monitoring GPIO pins for muscle activity.
    #[cfg(feature = "pi")]
    {
        let maestro_tx = manager_channel_map
            .get(Resource::Maestro.as_str_name())
            .clone();
        tokio::spawn(async move {
            start_monitoring_pin(maestro_tx).await;
        });

        let emg_tx = manager_channel_map // todo
            .get(Resource::Emg.as_str_name())
            .clone();
        tokio::spawn(async move {
            start_monitoring_pin(emg_tx).await;
        });


    }
    // Start the main TCP server loop to handle incoming connections.
    server::init(manager_channel_map).await;
}
