use core::task;

// All tasks operating on the BMS (Battery Management System) live in this file
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
use crate::request::TaskData::BmsData;
use crate::run_task;
use crate::sgcp::bms::*;
use crate::todo;
use crate::verify_channel_data;

/// Represents a BMS resource
pub struct Bms {
    // TODO: @krarpit Implement BMS interface
}

impl Resource for Bms {
    fn init() -> Self {
        Bms {} // stub
    }
}

impl ResourceManager for Manager<Bms> {
    fn tx(&self) -> Sender<ManagerChannelData> {
        self.tx.clone()
    }

    /// Handles all BMS-related tasks
    fn handle_task(&self, rcvd: ManagerChannelData) -> Result<()> {
        let data = verify_channel_data!(rcvd, Task, BmsData).map_err(|err: Error| err)?;
        let task = data.0;
        let task_data = data.1;
        let send_channel = rcvd.resp_tx;
        match task {
            Task::UndefinedTask => todo!(),
            Task::GetHealthMetrics => todo!(),
        }
        send_channel.send(TASK_SUCCESS.to_string());
        Ok(())
    }

    async fn run(&mut self) {
        run_task!(self, handle_task);
    }
}
