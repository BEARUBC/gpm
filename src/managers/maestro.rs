// All tasks operating on the Maestro servo controller live in this file.
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
use super::ResourceManager;
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use crate::not_on_pi;
use crate::request::TaskData::MaestroData;
use crate::run_task;
use crate::sgcp::maestro::*;
use crate::todo;
use crate::verify_channel_data;

pub struct Maestro {}
impl Resource for Maestro {}

// impl Maestro {
//     pub fn new() -> Self {
//         let (tx, mut rx) = channel(MAX_MPSC_CHANNEL_BUFFER);
//         #[cfg(feature = "pi")]
//         let mut controller: Maestro = Builder::default()
//             .baudrate(Baudrate::Baudrate11520)
//             .block_duration(Duration::from_millis(100))
//             .try_into()
//             .unwrap();
//         Maestro {
//             tx,
//             rx,
//             #[cfg(feature = "pi")]
//             controller,
//         }
//     }
// }

impl ResourceManager for Manager<Maestro> {
    fn init(&self) -> Result<()> {
        info!("Successfully initialized");
        Ok(())
    }

    fn tx(&self) -> Sender<ManagerChannelData> {
        self.tx.clone()
    }

    fn handle_task(&self, rcvd: ManagerChannelData) -> Result<()> {
        let data = verify_channel_data!(rcvd, Task, MaestroData).map_err(|err: Error| err)?;
        let task = data.0;
        let task_data = data.1;
        let send_channel = rcvd.resp_tx;
        let res = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined task type");
                "Undefined task, did you forget to initialize the message?".to_string()
            },
            Task::OpenFist => {
                #[cfg(not(feature = "pi"))]
                {
                    not_on_pi!();
                    "Successfully ran task!".to_string()
                }
                #[cfg(feature = "pi")]
                {
                    maestro.set_target(Channel::Channel0, MIN_QTR_PWM).unwrap();
                    maestro.set_target(Channel::Channel1, MIN_QTR_PWM).unwrap();
                    maestro.set_target(Channel::Channel2, MIN_QTR_PWM).unwrap();
                    "Successfully ran task!".to_string()
                }
            },
            Task::CloseFist => {
                #[cfg(not(feature = "pi"))]
                {
                    not_on_pi!();
                    "Successfully ran task!".to_string()
                }
                #[cfg(feature = "pi")]
                {
                    maestro.set_target(Channel::Channel0, MAX_QTR_PWM).unwrap();
                    maestro.set_target(Channel::Channel1, MAX_QTR_PWM).unwrap();
                    maestro.set_target(Channel::Channel2, MAX_QTR_PWM).unwrap();
                    "Successfully ran task!".to_string()
                }
            },
        };
        send_channel.send(res);
        Ok(())
    }

    async fn run(&mut self) {
        run_task!(self, handle_task);
    }
}
