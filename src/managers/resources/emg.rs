// All tasks operating on the EMG system live in this file
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::Resource;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::sgcp;
use crate::sgcp::emg::*;
use crate::todo;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;

/// Represents an EMG resource
pub struct Emg {}

impl Resource for Emg {
    fn init() -> Self {
        Emg {} // stub
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl ResourceManager for Manager<Emg> {
    type ResourceType = Emg;
    /// Handles all EMG-related tasks
    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, _, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;
        match task {
            Task::UndefinedTask => todo!(),
        }

        Ok(send_channel
            .send(TASK_SUCCESS.to_string())
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}
