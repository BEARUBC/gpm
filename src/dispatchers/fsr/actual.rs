use std::time::Duration;

use log::*;
use tokio::time::interval;

use super::FsrDispatcher;
use crate::ManagerChannelMap;
use crate::config::Config;
use crate::dispatchers::Dispatcher;
use crate::dispatchers::dispatch_task;
use crate::dispatchers::sgcp;


impl Dispatcher for FsrDispatcher {
    async fn run(manager_channel_map: ManagerChannelMap) {
        let fsr_config = Config::global()
            .dispatcher
            .fsr
            .as_ref()
            .expect("FSR config should be defined");
        let mut fsr_dispatcher = FsrDispatcher;

        let mut fsr_idle = interval(Duration::from_millis(fsr_config.pause_duration_ms));

        loop {
            fsr_dispatcher.dispatch(&manager_channel_map).await;
            _ = fsr_idle.tick();
        }
    }
}

impl FsrDispatcher {
    pub async fn dispatch(&mut self, manager_channel_map: &ManagerChannelMap) {
        let fsr_request = sgcp::Request {
            resource: sgcp::Resource::Fsr as i32,
            task_code: "IDLE".to_string(),
            task_data: None,
        };
        // send request to FSR
        let fsr_response = dispatch_task(fsr_request, manager_channel_map).await;
        match fsr_response {
            Ok(response) => {
                let maestro_request = sgcp::Request {
                    resource: sgcp::Resource::Maestro as i32,
                    task_code: response,
                    task_data: None,
                };

                // dispatch request to maestro
                match dispatch_task(maestro_request, manager_channel_map).await {
                    Ok(res) => info!("Task succeeded: {:?}", res),
                    Err(e) => error!("Task failed: {:?}", e),
                }
            },
            Err(e) => {
                error!("An error occurred when dispatching FSR task; error={e}");
                log::error!("An error occurred when dispatching FSR task; error={e}");
            },
        }
    }
}
