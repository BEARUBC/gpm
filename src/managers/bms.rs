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
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use super::TASK_SUCCESS;
use crate::parse_channel_data;
use crate::request::TaskData::BmsData;
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

impl Manager<Bms> {
    /// Handles all BMS-related tasks
    fn handle_task(&self, rcvd: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(rcvd, Task, BmsData).map_err(|e: Error| e)?;
        match task {
            Task::UndefinedTask => todo!(),
            Task::GetHealthMetrics => todo!(),
        }
        send_channel.send(TASK_SUCCESS.to_string());
        Ok(())
    }
}
