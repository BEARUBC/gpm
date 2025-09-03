use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use gpm::not_on_pi;
use gpm::sgcp::fsr::Task as FsrTask;
use gpm::sgcp::request::TaskData::FsrData;
use log::*;

use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::resources::fsr::Fsr;

impl ResourceManager for Manager<Fsr> {
    type ResourceType = Fsr;

    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, _task_data, send_channel) =
            parse_channel_data!(channel_data, FsrTask, FsrData).map_err(|e: Error| e)?;

        let task_result: Result<String, Error> = match task {
            FsrTask::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            },
            _ => {
                not_on_pi!();
                Ok(TASK_SUCCESS.to_string())
            },
        };

        let response = match task_result {
            Ok(message) => message,
            Err(e) => format!("Error: {e}"),
        };

        Ok(send_channel
            .send(response)
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}
