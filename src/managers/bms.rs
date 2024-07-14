// All tasks operating on the BMS system live in this file.
use anyhow::Result;
use tokio::sync::mpsc::{Sender, Receiver, channel};

use super::ResourceManager;

pub struct BMS {
    pub tx: Sender<(i32, tokio::sync::oneshot::Sender<std::string::String>)>,
    pub rx: Receiver<(i32, tokio::sync::oneshot::Sender<std::string::String>)>,
}

impl BMS {
    pub fn new() -> Self {
        let (tx, mut rx) = channel(32);
        BMS { tx, rx }
    }
}

impl ResourceManager for BMS {
    type Message = (i32, tokio::sync::oneshot::Sender<std::string::String>);

    fn init(&self) -> Result<()> {
        info!("Successfully initialized");
        Ok(()) // stub
    }

    fn tx(&self) -> Sender<Self::Message> {
        self.tx.clone()
    }

    fn handle_task(&self, task_code: i32) {
        // stub
    }

    async fn run(&mut self) {
        // stub
        info!("Listening for messages");
        while let Some(data) = self.rx.recv().await {
            info!("Recieved task_code={:?}", data.0);
            data.1.send("Successfully ran task!".to_string());
        }
    }
}
