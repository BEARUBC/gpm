// All tasks operating on the Maestro servo controller live in this file.
use anyhow::Result;
use tokio::sync::mpsc::{Sender, Receiver, channel};
use crate::sgcp::maestro::*;
use super::{ManagerChannelData, ResourceManager, Responder, MAX_MPSC_CHANNEL_BUFFER};
use log::*;


type MaestroMessage = (Task, Option<TaskData>);
pub type ChannelData = (MaestroMessage, Responder<std::string::String>);

pub struct Maestro {
    pub tx: Sender<ManagerChannelData>,
    pub rx: Receiver<ManagerChannelData>,
}

impl Maestro {
    pub fn new() -> Self {
        let (tx, mut rx) = channel(MAX_MPSC_CHANNEL_BUFFER);
        Maestro { tx, rx }
    }
}

impl ResourceManager for Maestro {
    fn init(&self) -> Result<()> {
        info!("Successfully initialized");
        Ok(()) // stub
    }

    fn tx(&self) -> Sender<ManagerChannelData> {
        self.tx.clone()
    }

    fn handle_task(&self, task_code: i32) {
        // stub
    }

    async fn run(&mut self) {
        // stub
        info!("Listening for messages");
        while let Some(data) = self.rx.recv().await { 
            match data {
                ManagerChannelData::MaestroChannelData(data) => {
                    info!("Recieved task_code={:?}", data.0);
                    data.1.send("Successfully ran task!".to_string());
                },
                _ => error!("Mismatched message type")
            }
        }
    }
}
