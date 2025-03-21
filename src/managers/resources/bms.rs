// All tasks operating on the BMS (Battery Management System) live in this file
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::Resource;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::BmsData;
use crate::sgcp;
use crate::sgcp::bms::*;
use crate::todo;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;

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
        let (task, _, send_channel) =
            parse_channel_data!(channel_data, Task, BmsData).map_err(|e: Error| e)?;
        match task {
            Task::UndefinedTask => todo!(),
            Task::GetHealthMetrics => todo!(),
        }
        Ok(send_channel
            .send(TASK_SUCCESS.to_string())
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}