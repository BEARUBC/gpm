use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;

use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::FsrData;
use crate::resources::fsr::Fsr;
use crate::sgcp::fsr::Task as FsrTask;

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
            FsrTask::Idle => {
                let vibrate_state = self.resource.process_data()?;

                if vibrate_state == 1 {
                    info!("Vibrate on");
                    Ok("VIBRATE_ON".to_string())
                } else {
                    info!("Vibrate off");
                    Ok("VIBRATE_OFF".to_string())
                }
            },
            FsrTask::Abort => {
                info!("Aborting FSR task");
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
