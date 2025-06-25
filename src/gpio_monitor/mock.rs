use crate::managers::ManagerChannelData;
use tokio::sync::mpsc::Sender;

pub async fn run_gpio_monitor_loop(_maestro_tx: Sender<ManagerChannelData>) {
    panic!("Cannot use GPIO monitor outside the Pi");
}
