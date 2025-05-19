// All tasks operating on the BMS (Battery Management System) live in this file
use anyhow::Error;
use anyhow::Result;
use log::*;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;

use super::MAX_MPSC_CHANNEL_BUFFER;
use super::Manager;
use super::ManagerChannelData;
use super::Resource;
use super::ResourceManager;
use super::Responder;
use super::TASK_SUCCESS;
use crate::parse_channel_data;
use crate::request::TaskData::BmsData;
use crate::sgcp;
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

    fn name() -> String {
        sgcp::Resource::Bms.as_str_name().to_string()
    }
}

impl ResourceManager for Manager<Bms> {
    type ResourceType = Bms;
    /// Handles all BMS-related tasks
    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(channel_data, Task, BmsData).map_err(|e: Error| e)?;
        match task {
            Task::UndefinedTask => todo!(),
            Task::GetHealthMetrics => todo!(),
        }
        send_channel.send(TASK_SUCCESS.to_string());
        Ok(())
    }
}
