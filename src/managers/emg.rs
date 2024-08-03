// All tasks operating on the EMG system live in this file.
use anyhow::Error;
use anyhow::Result;
use log::*;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::ManagerChannelData;
use super::ResourceManager;
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use crate::run_task;
use crate::sgcp::emg::*;
use crate::todo;
use crate::verify_channel_data;

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

    fn handle_task(&self, task_data: ManagerChannelData) -> Result<()> {
        let data = verify_channel_data!(task_data, ManagerChannelData::EmgChannelData)?;
        let task = data.0.0;
        let task_data = data.0.1;
        let send_channel = data.1;
        match task {
            Task::UndefinedTask => todo!(),
        }
        send_channel.send("Successfully ran task!".to_string());
        Ok(())
    }

    async fn run(&mut self) {
        run_task!(self, handle_task);
    }
}
