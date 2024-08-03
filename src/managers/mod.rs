mod bms;
mod emg;
mod maestro;

use anyhow::Result;
pub use bms::Bms;
pub use emg::Emg;
pub use maestro::Maestro;
use tokio::sync::mpsc::Sender;

use crate::async_match_managers;
use crate::match_managers;

// Represents the response data type from a task manager
type Responder<T> = tokio::sync::oneshot::Sender<T>;

const MAX_MPSC_CHANNEL_BUFFER: usize = 32;

pub trait ResourceManager {
    async fn run(&mut self);
    fn init(&self) -> Result<()>;
    fn tx(&self) -> Sender<ManagerChannelData>;
    fn handle_task(&self, task_data: ManagerChannelData) -> Result<()>;
}

pub enum Manager {
    BmsManager(Bms),
    EmgManager(Emg),
    MaestroManager(Maestro),
}

#[derive(Debug)]
pub enum ManagerChannelData {
    BmsChannelData(bms::ChannelData),
    EmgChannelData(emg::ChannelData),
    MaestroChannelData(maestro::ChannelData),
}

impl ResourceManager for Manager {
    fn init(&self) -> Result<()> {
        match_managers!(self, init)
    }

    fn tx(&self) -> Sender<ManagerChannelData> {
        match_managers!(self, tx)
    }

    fn handle_task(&self, task_data: ManagerChannelData) -> Result<()> {
        match_managers!(self, handle_task, task_data)
    }

    async fn run(&mut self) {
        async_match_managers!(self, run)
    }
}
