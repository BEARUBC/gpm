mod config;
mod dispatchers;
mod macros;
mod managers;
mod resources;
mod telemetry;

use config::CommandDispatchStrategy;
use config::Config;
use log::*;
use managers::HasMpscChannel;
use managers::Manager;
use managers::ManagerChannelData;
use managers::ResourceManager;
use resources::bms::Bms;
use resources::emg::Emg;
use resources::maestro::Maestro;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

// Import protobuf definitions for task communication
import_sgcp!();

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

    tokio::spawn(async {
        let mut exporter = telemetry::Exporter::new();
        exporter.init().await
    });

    info!(
        "Using {:?} as the command dispatch strategy",
        Config::global().command_dispatch_strategy
    );

    match Config::global().command_dispatch_strategy {
        CommandDispatchStrategy::Tcp => {
            dispatchers::tcp::run_tcp_dispatcher_loop(manager_channel_map).await
        },
        CommandDispatchStrategy::Gpio => {
            dispatchers::gpio::run_gpio_dispatcher_loop(manager_channel_map).await
        },
        CommandDispatchStrategy::Emg => {
            dispatchers::emg::run_emg_dispatcher_loop(manager_channel_map).await
        },
    }
}
