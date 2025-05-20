mod config;
mod gpio_monitor;
mod macros;
mod managers;
mod server;
mod telemetry;

use config::CommandDispatchStrategy;
use config::Config;
use log::*;
use managers::HasMpscChannel;
use managers::Manager;
use managers::ManagerChannelData;
use managers::ResourceManager;
use managers::resources::bms::Bms;
use managers::resources::emg::Emg;
use managers::resources::maestro::Maestro;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

// Import protobuf definitions for task communication
import_sgcp!();

/// Main entry point for the bionic arm system.
/// Initializes all resource managers, telemetry, and starts the TCP server.
#[tokio::main]
async fn main() {
    #[cfg(feature = "dev")]
    console_subscriber::init(); // Used for Tokio runtime diagnostics
    config::logger_init();

    // Initialize resource managers and their communication channels.
    let manager_channel_map = managers::macros::init_resource_managers! {
        sgcp::Resource::Bms => Manager::<Bms>::new(),
        sgcp::Resource::Emg => Manager::<Emg>::new(),
        sgcp::Resource::Maestro => Manager::<Maestro>::new()
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
                    .get(sgcp::Resource::Maestro.as_str_name())
                    .expect("Expected the Maestro manager to be initialized")
                    .clone(),
            )
            .await;
        },
    }
}
