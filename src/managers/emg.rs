// All tasks operating on the EMG system live in this file
// NOTE: All EMG proccessing will now be handled on the Jetson Nano instead of GPM. Leaving
//       this file here in case we decide to implement an EMG interface for GPM anyway.
use anyhow::Error;
use anyhow::Result;
use log::*;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::Manager;
use super::ManagerChannelData;
use super::Resource;
use super::ResourceManager;
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use super::TASK_SUCCESS;
use crate::request::TaskData::EmgData;
use crate::run_task;
use crate::sgcp::emg::*;
use crate::todo;
use crate::verify_channel_data;

/// Represents an EMG resource
pub struct Emg {
    // WILL NOT IMPLEMENT (SEE NOTE AT TOP OF FILE)
}

impl Resource for Emg {
    fn init() -> Self {
        Emg {} // stub
    }
}

impl ResourceManager for Manager<Emg> {
    fn tx(&self) -> Sender<ManagerChannelData> {
        self.tx.clone()
    }

    /// Handles all EMG-related tasks
    fn handle_task(&self, rcvd: ManagerChannelData) -> Result<()> {
        let data = verify_channel_data!(rcvd, Task, EmgData).map_err(|err: Error| err)?;
        let task = data.0;
        let task_data = data.1;
        let send_channel = rcvd.resp_tx;
        match task {
            Task::UndefinedTask => todo!(),
        }
        send_channel.send(TASK_SUCCESS.to_string());
        Ok(())
    }

    async fn run(&mut self) {
        run_task!(self, handle_task);
    }
}
