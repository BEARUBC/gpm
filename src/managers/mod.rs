mod bms;
mod emg;
mod maestro;

use anyhow::Result;
pub use bms::Bms;
pub use emg::Emg;
pub use maestro::Maestro;
use tokio::sync::mpsc::Sender;

type Responder<T> = tokio::sync::oneshot::Sender<T>;

const MAX_MPSC_CHANNEL_BUFFER: usize = 32;

pub trait ResourceManager {
    async fn run(&mut self);
    fn init(&self) -> Result<()>;
    fn tx(&self) -> Sender<ManagerChannelData>;
    fn handle_task(&self, task_code: i32);
}

pub enum Manager {
    BmsManager(Bms),
    EmgManager(Emg),
    MaestroManager(Maestro),
}

pub enum ManagerChannelData {
    BmsChannelData(bms::ChannelData),
    EmgChannelData(emg::ChannelData),
    MaestroChannelData(maestro::ChannelData),
}

impl ResourceManager for Manager {
    fn init(&self) -> Result<()> {
        match self {
            Manager::BmsManager(bms) => bms.init(),
            Manager::EmgManager(emg) => emg.init(),
            Manager::MaestroManager(maestro) => maestro.init(),
        }
    }

    fn tx(&self) -> Sender<ManagerChannelData> {
        match self {
            Manager::BmsManager(bms) => bms.tx(),
            Manager::EmgManager(emg) => emg.tx(),
            Manager::MaestroManager(maestro) => maestro.tx(),
        }
    }

    fn handle_task(&self, task_code: i32) {
        // stub
    }

    async fn run(&mut self) {
        match self {
            Manager::BmsManager(bms) => bms.run().await,
            Manager::EmgManager(emg) => emg.run().await,
            Manager::MaestroManager(maestro) => maestro.run().await,
        }
    }
}
