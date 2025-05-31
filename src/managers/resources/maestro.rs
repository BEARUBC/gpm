// All tasks operating on the Maestro servo controller live in this file

#![allow(unused_imports)] // Silence warnings because of cfg-gated code

use crate::managers::macros::parse_channel_data;
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::Resource;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::not_on_pi;
use crate::request::TaskData::MaestroData;
use crate::sgcp;
use crate::sgcp::maestro::Task as MaestroTask;
use crate::sgcp::maestro::*;
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use log::*;
#[cfg(feature = "pi")]
use raestro::maestro::{
    builder::Builder,
    constants::{Baudrate, Channel, MAX_QTR_PWM, MIN_QTR_PWM},
};
use std::time::Duration;

/// Represents a Maestro resource
pub struct Maestro {
    #[cfg(feature = "pi")]
    controller: raestro::maestro::Maestro,
}

impl Resource for Maestro {
    fn init() -> Self {
        #[cfg(feature = "pi")]
        {
            let mut controller: raestro::maestro::Maestro = Builder::default()
                .baudrate(Baudrate::Baudrate11520)
                .block_duration(Duration::from_millis(100))
                .try_into()
                .expect("Could not initialize Raestro");
            Maestro { controller }
        }
        #[cfg(not(feature = "pi"))]
        Maestro {}
    }

    fn name() -> String {
        sgcp::Resource::Maestro.as_str_name().to_string()
    }
}

impl ResourceManager for Manager<Maestro> {
    type ResourceType = Maestro;

    /// Handles all Maestro-related tasks
    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, _, send_channel) =
            parse_channel_data!(channel_data, MaestroTask, MaestroData).map_err(|e: Error| e)?;

        #[cfg(feature = "pi")]
        let controller = &mut self.resource.controller;

        let task_result = match task {
            MaestroTask::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            },
            MaestroTask::OpenFist => {
                #[cfg(not(feature = "pi"))]
                {
                    not_on_pi!();
                    Ok(())
                }
                #[cfg(feature = "pi")]
                {
                    controller.set_target(Channel::Channel0, MIN_QTR_PWM)?;
                    controller.set_target(Channel::Channel1, MIN_QTR_PWM)?;
                    controller.set_target(Channel::Channel2, MIN_QTR_PWM)?;
                    Ok(())
                }
            },
            MaestroTask::CloseFist => {
                #[cfg(not(feature = "pi"))]
                {
                    not_on_pi!();
                    Ok(())
                }
                #[cfg(feature = "pi")]
                {
                    controller.set_target(Channel::Channel0, MAX_QTR_PWM)?;
                    controller.set_target(Channel::Channel1, MAX_QTR_PWM)?;
                    controller.set_target(Channel::Channel2, MAX_QTR_PWM)?;
                    Ok(())
                }
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
