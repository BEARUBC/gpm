// All tasks operating on the EMG system live in this file.
use anyhow::Result;
use log::*;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::ManagerChannelData;
use super::ResourceManager;
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use crate::sgcp::emg::*;

pub type ChannelData = (EmgMessage, Responder<std::string::String>);

type EmgMessage = (Task, Option<TaskData>);

pub struct Emg {
    pub tx: Sender<ManagerChannelData>,
    pub rx: Receiver<ManagerChannelData>,
}

impl Emg {
    pub fn new() -> Self {
        let (tx, mut rx) = channel(MAX_MPSC_CHANNEL_BUFFER);
        Emg { tx, rx }
    }
}

impl ResourceManager for Emg {
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
                ManagerChannelData::EmgChannelData(data) => {
                    info!("Recieved task_code={:?}", data.0);
                    data.1.send("Successfully ran task!".to_string());
                },
                _ => error!("Mismatched message type"),
            }
        }
    }
}
