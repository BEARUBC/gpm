use crate::{ManagerChannelMap, dispatchers::Dispatcher};

use super::GpioDispatcher;

impl Dispatcher for GpioDispatcher {
    async fn run(_manager_channel_map: ManagerChannelMap) {
        panic!("Cannot use GPIO monitor outside the Pi");
    }
}
