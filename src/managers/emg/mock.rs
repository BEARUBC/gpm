use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::resources::emg::Emg;
use crate::sgcp::emg::*;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use gpm::not_on_pi;
use log::*;

impl ResourceManager for Manager<Emg> {
    type ResourceType = Emg;

    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, _, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;

        let res = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            },
            Task::Idle => {
                not_on_pi!();
                Ok(TASK_SUCCESS.to_string())
            },
            Task::Calibrate => {
                not_on_pi!();
                Ok(TASK_SUCCESS.to_string())
            },
            Task::Abort => {
                not_on_pi!();
                Ok(TASK_SUCCESS.to_string())
            },
        };

        let response = match res {
            Ok(message) => {
                if message == "OPEN HAND" || message == "CLOSE HAND" {
                    message
                } else {
                    TASK_SUCCESS.to_string()
                }
            },
            Err(e) => format!("Error: {e}"),
        };

        Ok(send_channel
            .send(response)
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}
