// All tasks operating on the Maestro servo controller live in this file
use anyhow::Error;
use anyhow::Result;
use log::*;
#[cfg(feature = "pi")]
use raestro::maestro::builder::Builder;
#[cfg(feature = "pi")]
use raestro::maestro::constants::Baudrate;
#[cfg(feature = "pi")]
use raestro::maestro::constants::Channel;
#[cfg(feature = "pi")]
use raestro::maestro::constants::MAX_QTR_PWM;
#[cfg(feature = "pi")]
use raestro::maestro::constants::MIN_QTR_PWM;
#[cfg(feature = "pi")]
use raestro::maestro::Maestro;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::Manager;
use super::ManagerChannelData;
use super::Resource;
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use crate::managers::TASK_SUCCESS;
use crate::managers::UNDEFINED_TASK;
use crate::not_on_pi;
use crate::parse_channel_data;
use crate::request::TaskData::MaestroData;
use crate::sgcp::maestro::*;
use crate::todo;
use crate::verify_channel_data;

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
                .unwrap();
            Maestro { controller }
        }
        #[cfg(not(feature = "pi"))]
        Maestro {}
    }
}

impl Manager<Maestro> {
    /// Handles all Maestro-related tasks
    fn handle_task(&self, rcvd: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(rcvd, Task, MaestroData).map_err(|e: Error| e)?;
        let res = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined task type");
                UNDEFINED_TASK.to_string()
            },
            Task::OpenFist => {
                #[cfg(not(feature = "pi"))]
                {
                    not_on_pi!();
                    TASK_SUCCESS.to_string()
                }
                #[cfg(feature = "pi")]
                {
                    self.metadata
                        .controller
                        .set_target(Channel::Channel0, MIN_QTR_PWM)
                        .unwrap();
                    self.metadata
                        .controller
                        .set_target(Channel::Channel1, MIN_QTR_PWM)
                        .unwrap();
                    self.metadata
                        .controller
                        .set_target(Channel::Channel2, MIN_QTR_PWM)
                        .unwrap();
                    TASK_SUCCESS.to_string()
                }
            },
            Task::CloseFist => {
                #[cfg(not(feature = "pi"))]
                {
                    not_on_pi!();
                    TASK_SUCCESS.to_string()
                }
                #[cfg(feature = "pi")]
                {
                    self.metadata
                        .controller
                        .set_target(Channel::Channel0, MAX_QTR_PWM)
                        .unwrap();
                    self.metadata
                        .controller
                        .set_target(Channel::Channel1, MAX_QTR_PWM)
                        .unwrap();
                    self.metadata
                        .controller
                        .set_target(Channel::Channel2, MAX_QTR_PWM)
                        .unwrap();
                    TASK_SUCCESS.to_string()
                }
            },
        };
        send_channel.send(res);
        Ok(())
    }
}
