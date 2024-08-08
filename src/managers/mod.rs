mod bms;
mod emg;
mod maestro;

use anyhow::Error;
use anyhow::Result;
pub use bms::Bms;
pub use emg::Emg;
use log::error;
use log::info;
pub use maestro::Maestro;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use crate::request::TaskData;

/// Represents the channel used by a resource manager to return the task response
type Responder<T> = tokio::sync::oneshot::Sender<T>;

// General resource manager configs
const MAX_MPSC_CHANNEL_BUFFER: usize = 32;

// Resource manager return values
const TASK_SUCCESS: &str = "Successfully ran task";
const UNDEFINED_TASK: &str = "Undefined task, did you forget to initialize the message?";

/// Represent a resource manager
pub trait ResourceManager {
    async fn run(&mut self);
    fn handle_task(&self, data: ManagerChannelData) -> Result<()>;
}

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}

/// Represents a resource manager
pub struct Manager<S: Resource> {
    pub tx: Sender<ManagerChannelData>,
    pub rx: Receiver<ManagerChannelData>,
    /// Holds resource specific metadata
    metadata: S,
}

impl<S: Resource> Manager<S> {
    pub fn new() -> Self {
        let (tx, mut rx) = channel(MAX_MPSC_CHANNEL_BUFFER);
        Manager::<S> {
            tx,
            rx,
            metadata: S::init(),
        }
    }

    /// Returns tx component of the resource manager's MPSC channel to
    /// enable sending tasks
    pub fn tx(&self) -> Sender<ManagerChannelData> {
        self.tx.clone()
    }
}

/// Represents the format of messages that will be sent to each resource manager.
/// Note that it is the resource manager's responsibility to perform necessary
/// validation on the received data.
#[derive(Debug)]
pub struct ManagerChannelData {
    pub task_code: String,
    pub task_data: Option<TaskData>,
    pub resp_tx: Responder<String>,
}
