use super::FsrDispatcher;
use crate::ManagerChannelMap;
use crate::dispatchers::Dispatcher;

impl Dispatcher for FsrDispatcher {
    async fn run(_manager_channel_map: ManagerChannelMap) {
        panic!("Cannot run FSR monitor loop outside the Pi");
    }
}
