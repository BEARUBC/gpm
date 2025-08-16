use super::BioSignalDispatcher;
use crate::ManagerChannelMap;
use crate::dispatchers::Dispatcher;

impl Dispatcher for BioSignalDispatcher {
    async fn run(_manager_channel_map: ManagerChannelMap) {
        panic!("Cannot run Bio Signal monitor loop outside the Pi");
    }
}
