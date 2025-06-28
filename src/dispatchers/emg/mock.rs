use crate::{ManagerChannelMap, dispatchers::Dispatcher};

use super::EmgDispatcher;

impl Dispatcher for EmgDispatcher {
    async fn run(_manager_channel_map: ManagerChannelMap) {
        panic!("Cannot run EMG monitor loop outside the Pi");
    }
}
