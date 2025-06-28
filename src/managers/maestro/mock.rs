use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::MaestroData;
use crate::resources::maestro::Maestro;
use crate::sgcp::maestro::Task as MaestroTask;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use gpm::not_on_pi;
use log::*;

impl ResourceManager for Manager<Maestro> {
    type ResourceType = Maestro;

    /// Handles all Maestro-related tasks
    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, _, send_channel) =
            parse_channel_data!(channel_data, MaestroTask, MaestroData).map_err(|e: Error| e)?;

        let task_result = match task {
            MaestroTask::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            },
            MaestroTask::OpenFist => {
                not_on_pi!();
                Ok(())
            },
            MaestroTask::CloseFist => {
                not_on_pi!();
                Ok(())
            },
        };

        let response = match task_result {
            Ok(_) => TASK_SUCCESS.to_string(),
            Err(e) => format!("Error: {e}"),
        };

        Ok(send_channel
            .send(response)
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}
