mod bms;
mod emg;
mod maestro;

use anyhow::Result;
pub use bms::Bms;
pub use emg::Emg;
pub use maestro::Maestro;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
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

pub struct Manager<S: Resource> {
    pub tx: Sender<ManagerChannelData>,
    pub rx: Receiver<ManagerChannelData>,
    // This reassures the compiler that type S
    // is used
    marker: std::marker::PhantomData<S>,
}

impl<S: Resource> Manager<S> {
    pub fn new() -> Self {
        let (tx, mut rx) = channel(MAX_MPSC_CHANNEL_BUFFER);
        Manager::<S> {
            tx,
            rx,
            marker: std::marker::PhantomData::<S>,
        }
    }
}

pub trait Resource {}

#[derive(Debug)]
pub enum ManagerChannelData {
    BmsChannelData(bms::ChannelData),
    EmgChannelData(emg::ChannelData),
    MaestroChannelData(maestro::ChannelData),
}
