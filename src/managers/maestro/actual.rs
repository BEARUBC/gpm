use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::MaestroData;
use crate::resources::Resource;
use crate::resources::maestro::Maestro;
use crate::sgcp;
use crate::sgcp::maestro::Task as MaestroTask;
use crate::sgcp::maestro::*;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;
use raestro::maestro::{
    builder::Builder,
    constants::{Baudrate, Channel, MAX_QTR_PWM, MIN_QTR_PWM},
};
use std::time::Duration;

impl ResourceManager for Manager<Maestro> {
    type ResourceType = Maestro;

    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, _, send_channel) =
            parse_channel_data!(channel_data, MaestroTask, MaestroData).map_err(|e: Error| e)?;

        let controller = &mut self.resource.controller;

        let task_result = match task {
            MaestroTask::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            },
            MaestroTask::OpenFist => {
                controller.set_target(Channel::Channel0, MIN_QTR_PWM)?;
                controller.set_target(Channel::Channel1, MIN_QTR_PWM)?;
                controller.set_target(Channel::Channel2, MIN_QTR_PWM)?;
                Ok(())
            },
            MaestroTask::CloseFist => {
                controller.set_target(Channel::Channel0, MAX_QTR_PWM)?;
                controller.set_target(Channel::Channel1, MAX_QTR_PWM)?;
                controller.set_target(Channel::Channel2, MAX_QTR_PWM)?;
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
