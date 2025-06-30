pub mod emg;
pub mod gpio;
mod macros;
pub mod tcp;

use crate::ManagerChannelMap;
use crate::managers::ManagerChannelData;
use crate::sgcp;
use anyhow::Error;
use anyhow::Result;
use log::info;
use tokio::sync::oneshot;

pub trait Dispatcher {
    async fn run(manager_channel_map: ManagerChannelMap);
}

/// Dispatches a request to the appropiate resource manager. Returns the response from the task.
pub async fn dispatch_task(
    request: sgcp::Request,
    manager_channel_map: &ManagerChannelMap,
) -> Result<String> {
    macros::dispatch_task!(
        request,
        sgcp::Resource::Bms => manager_channel_map.get(sgcp::Resource::Bms.as_str_name()),
        sgcp::Resource::Emg => manager_channel_map.get(sgcp::Resource::Emg.as_str_name()),
        sgcp::Resource::Maestro => manager_channel_map.get(sgcp::Resource::Maestro.as_str_name())
    )
}
