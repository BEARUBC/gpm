mod config;
mod dispatchers;
mod exporters;
mod managers;
mod resources;

use std::collections::HashMap;
use std::sync::{Arc};
use config::CommandDispatchStrategy;
use config::Config;
use dispatchers::Dispatcher;
use dispatchers::bio_signal::BioSignalDispatcher;
use dispatchers::emg::EmgDispatcher;
use dispatchers::fsr::FsrDispatcher;
use dispatchers::gpio::GpioDispatcher;
use dispatchers::tcp::TcpDispatcher;
use gpm::sgcp::bms;
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
use tokio::sync::Mutex;


/// Represents the mapping between resource manager keys and the tx component
/// of the resource manager's MPSC channel
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;

#[tokio::main]
async fn main() {
    #[cfg(feature = "dev")]
    console_subscriber::init(); // Used for Tokio runtime diagnostics
    config::logger_init();

    //init managers outside of macro so we can pass it to the exporter
    let bms_manager = Arc::new(Mutex::new(Manager::<Bms>::new()));
    let emg_manager = Arc::new(Mutex::new(Manager::<Emg>::new()));
    let maestro_manager = Arc::new(Mutex::new(Manager::<Maestro>::new()));
    let fsr_manager = Arc::new(Mutex::new(Manager::<Fsr>::new()));

    // Initialize resource managers and their communication channels.
    let manager_channel_map = managers::macros::init_resource_managers! {
        gpm::sgcp::Resource::Bms => Arc::clone(&bms_manager),
        gpm::sgcp::Resource::Emg => Arc::clone(&emg_manager),
        gpm::sgcp::Resource::Maestro => Arc::clone(&maestro_manager),
        gpm::sgcp::Resource::Fsr => Arc::clone(&fsr_manager)
    };

    tokio::spawn(async {
        let mut exporter = exporters::prometheus::Exporter::new();
        exporter.init().await
    });

    tokio::spawn(async {
        let exporter = exporters::emg::Exporter::new(emg_manager);
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
