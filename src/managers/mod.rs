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

use crate::request::TaskData;

/// Represents the channel used by a resource manager to return the task response
type Responder<T> = tokio::sync::oneshot::Sender<T>;

// General resource manager configs
const MAX_MPSC_CHANNEL_BUFFER: usize = 32;

// Resource manager return values
const TASK_SUCCESS: &str = "Successfully ran task";
const UNDEFINED_TASK: &str = "Undefined task, did you forget to initialize the message?";

pub trait ResourceManager {
    /// Starts the resource manager listener loop
    async fn run(&mut self);
    /// Returns tx component of the resource manager's MPSC channel to
    /// enable sending tasks
    fn tx(&self) -> Sender<ManagerChannelData>;
    /// Handles every possible type of task for its resource
    fn handle_task(&self, task_data: ManagerChannelData) -> Result<()>;
}

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
}

pub trait Resource {
    /// Initializes the resource
    fn init() -> Self;
}

/// Represents the format of messages that will sent to each resource manager.
/// Note that it is the resource manager's responsibility to perform necessary
/// validation on the received data.
#[derive(Debug)]
pub struct ManagerChannelData {
    pub task_code: String,
    pub task_data: Option<TaskData>,
    pub resp_tx: Responder<String>,
}
