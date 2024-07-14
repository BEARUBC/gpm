use anyhow::Result;
use tokio::sync::mpsc::Sender;

pub mod bms;
pub mod emg;
pub mod maestro;

pub trait ResourceManager {
    type Message: 'static + Send + Sync;

    async fn run(&mut self);
    fn init(&self) -> Result<()>;
    fn tx(&self) -> Sender<Self::Message>;
    fn handle_task(&self, task_code: i32);
}