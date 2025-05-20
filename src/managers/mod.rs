pub mod macros;
pub mod resources;

use crate::request::TaskData;
use anyhow::Result;
use log::error;
use log::info;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;

/// Represents the channel used by a resource manager to return the task response
type Responder<T> = tokio::sync::oneshot::Sender<T>;

// General resource manager configs
const MAX_MPSC_CHANNEL_BUFFER: usize = 32;

// Resource manager return values
const TASK_SUCCESS: &str = "Successfully ran task";

/// Represent a resource manager
pub trait ResourceManager: HasMpscChannel {
    type ResourceType: Resource;

    async fn handle_task(&mut self, data: ManagerChannelData) -> Result<()>;

    async fn run(&mut self) {
        info!(
            "{:?} resource manager now listening for messages",
            Self::ResourceType::name()
        );
        while let Some(data) = self.rx().recv().await {
            match self.handle_task(data).await {
                Err(err) => error!(
                    "Handling {:?} task failed with error={:?}",
                    Self::ResourceType::name(),
                    err
                ),
                _ => (),
            };
        }
    }
}

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}

pub trait HasMpscChannel {
    fn tx(&self) -> Sender<ManagerChannelData>;
    fn rx(&mut self) -> &mut Receiver<ManagerChannelData>;
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
}

impl<S: Resource> HasMpscChannel for Manager<S> {
    /// Returns tx component of the resource manager's MPSC channel to
    /// enable sending tasks
    fn tx(&self) -> Sender<ManagerChannelData> {
        self.tx.clone()
    }

    /// Returns rx component of the resource manager's MPSC channel to
    /// enable reading responses
    fn rx(&mut self) -> &mut Receiver<ManagerChannelData> {
        &mut self.rx
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
