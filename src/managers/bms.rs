// All tasks operating on the BMS system live in this file.
use anyhow::Result;
use log::*;
use std::path::Component;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use super::{ManagerChannelData, ResourceManager, Responder, MAX_MPSC_CHANNEL_BUFFER};
use crate::sgcp::bms::*;

type BmsMessage = (Task, Option<TaskData>);
pub type ChannelData = (BmsMessage, Responder<std::string::String>);

pub struct Bms {
    pub tx: Sender<ManagerChannelData>,
    pub rx: Receiver<ManagerChannelData>,
}

impl Bms {
    pub fn new() -> Self {
        let (tx, mut rx) = channel(MAX_MPSC_CHANNEL_BUFFER);
        Bms { tx, rx }
    }
}

impl ResourceManager for Bms {
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
                ManagerChannelData::BmsChannelData(data) => {
                    info!("Recieved task_code={:?}", data.0);
                    data.1.send("Successfully ran task!".to_string());
                }
                _ => error!("Mismatched message type"),
            }
        }
    }
}
