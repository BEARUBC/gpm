mod config;
mod dispatchers;
mod exporters;
mod managers;
mod resources;

use std::collections::HashMap;

use config::CommandDispatchStrategy;
use config::Config;
use dispatchers::Dispatcher;
use dispatchers::bio_signal::BioSignalDispatcher;
use dispatchers::emg::EmgDispatcher;
use dispatchers::fsr::FsrDispatcher;
use dispatchers::gpio::GpioDispatcher;
use dispatchers::tcp::TcpDispatcher;
use log::*;
use managers::HasMpscChannel;
use managers::Manager;
use managers::ManagerChannelData;
use managers::ResourceManager;
use resources::bms::Bms;
use resources::emg::Emg;
use resources::fsr::Fsr;
use resources::maestro::Maestro;
use tokio::sync::mpsc::Sender;


/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

#[tokio::main]
async fn main() {
    #[cfg(feature = "dev")]
    console_subscriber::init(); // Used for Tokio runtime diagnostics
    config::logger_init();

    // Initialize resource managers and their communication channels.
    let manager_channel_map = managers::macros::init_resource_managers! {
        sgcp::Resource::Bms => Manager::<Bms>::new(),
        sgcp::Resource::Emg => Manager::<Emg>::new(),
        sgcp::Resource::Maestro => Manager::<Maestro>::new(),
        sgcp::Resource::Fsr => Manager::<Fsr>::new()
    };

    tokio::spawn(async {
        let mut exporter = exporters::prometheus::Exporter::new();
        exporter.init().await
    });

    tokio::spawn(async {
        let exporter = exporters::emg::Exporter::new();
        exporter.init().await
    });

    info!(
        "Using {:?} as the command dispatch strategy",
        Config::global().command_dispatch_strategy
    );

    match Config::global().command_dispatch_strategy {
        CommandDispatchStrategy::Tcp => TcpDispatcher::run(manager_channel_map).await,
        CommandDispatchStrategy::Gpio => GpioDispatcher::run(manager_channel_map).await,
        CommandDispatchStrategy::BioSignal => BioSignalDispatcher::run(manager_channel_map).await,
        CommandDispatchStrategy::Emg => EmgDispatcher::run(manager_channel_map).await,
        CommandDispatchStrategy::Fsr => FsrDispatcher::run(manager_channel_map).await,
    }
}
