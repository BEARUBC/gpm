#![allow(warnings)]

// This file contains the main TCP connection loop
// It is responsible for handling incoming TCP connections, delegating tasks to resource managers, and initializing key components.
mod config; // Configuration settings (e.g., TCP address, buffer sizes)
mod connection; // Handles TCP connection framing and data transmission
mod gpio_monitor;
mod telemetry; // Telemetry exporter for system metrics // Provides an alternate strategy for dispatching commands based on GPIO pin
// state
mod macros; // Utility macros for common functionality
mod managers; // Resource management framework
mod server; // Main server loop and task routing

use std::any::Any;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result;
use bytes::BytesMut;
use config::CommandDispatchStrategy;
use config::Config;
use connection::Connection;
use log::*;
use managers::Bms;
use managers::Emg;
use managers::Maestro;
use managers::ManagerChannelData;
use prost::Message;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::time::sleep;

use crate::managers::Manager;
use crate::managers::ResourceManager;

/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

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

/// Main entry point for the bionic arm system.
/// Initializes all resource managers, telemetry, and starts the TCP server.
#[tokio::main]
async fn main() {
    #[cfg(feature = "dev")]
    console_subscriber::init(); // Used for Tokio runtime diagnostics
    config::logger_init();

    // Initialize resource managers and their communication channels.
    let manager_channel_map = init_resource_managers! {
        Resource::Bms => Manager::<Bms>::new(),
        Resource::Emg => Manager::<Emg>::new(),
        Resource::Maestro => Manager::<Maestro>::new()
    };

    // Spawn the telemetry exporter as an independent async task.
    tokio::spawn(async {
        let mut exporter = telemetry::Exporter::new();
        exporter.init().await
    });

    match Config::global().command_dispatch_strategy {
        CommandDispatchStrategy::Server => server::run_server_loop(manager_channel_map).await,
        CommandDispatchStrategy::GpioMonitor => {
            gpio_monitor::run_gpio_monitor_loop(
                manager_channel_map
                    .get(Resource::Maestro.as_str_name())
                    .expect("Expected the Maestro manager to be initialized")
                    .clone(),
            )
            .await;
        },
    }
}
