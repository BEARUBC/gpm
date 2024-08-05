// All tasks operating on the EMG system live in this file.
use anyhow::Error;
use anyhow::Result;
use log::*;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::Manager;
use super::ManagerChannelData;
use super::Resource;
use super::ResourceManager;
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use crate::run_task;
use crate::sgcp::emg::*;
use crate::todo;
use crate::verify_channel_data;

pub type ChannelData = (EmgMessage, Responder<std::string::String>);

type EmgMessage = (Task, Option<TaskData>);

pub struct Emg {}
impl Resource for Emg {}

impl ResourceManager for Manager<Emg> {
    fn init(&self) -> Result<()> {
        info!("Successfully initialized");
        Ok(()) // stub
    }

    fn tx(&self) -> Sender<ManagerChannelData> {
        self.tx.clone()
    }

    fn handle_task(&self, rcvd: ManagerChannelData) -> Result<()> {
        let _data: Result<(Task, Option<TaskData>), Error> =
            verify_channel_data!(rcvd, Task, crate::request::TaskData::EmgData);
        let data = _data?;
        let task = data.0;
        let task_data = data.1;
        let send_channel = rcvd.resp_tx;
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
