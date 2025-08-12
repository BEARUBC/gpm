use std::time::Duration;

use gpm::sgcp;
use log::*;
use tokio::time::interval;

use super::EmgDispatcher;
use crate::ManagerChannelMap;
use crate::config::Config;
use crate::dispatchers::Dispatcher;
use crate::dispatchers::dispatch_task;

// TODO: refactor
impl Dispatcher for EmgDispatcher {
    async fn run(manager_channel_map: ManagerChannelMap) {
        let emg_config = Config::global()
            .dispatcher
            .emg
            .as_ref()
            .expect("Expected EMG config to be defined");

        let mut emg_idle = interval(Duration::from_millis(emg_config.sampling_speed_ms)); // 1000 ms for 1 Hz sampling rate for idle tasks, 2 ms for 500 Hz sampling rate

        let mut emg_dispatcher = EmgDispatcher;
        // let mut HAPTICS_idle = interval(Duration::from_millis(1000)); // 1 Hz sampling rate //
        // example for haptics
        let send_channel_map = manager_channel_map.clone();
        loop {
            emg_dispatcher.process_idle_task(&send_channel_map).await;

            _ = emg_idle.tick();
        }
        // loop {
        //     tokio::select! {
        //         _ = emg_idle.tick() => {
        //             process_idle_task(&send_channel_map, sgcp::Resource::Emg, "IDLE",
        // &emg_response_mapping).await;         }
        //         // _ = HAPTICS_idle.tick() => {
        //         //     // handle haptics idle task here
        //         // }
        //     }
        // }
    }
}

impl EmgDispatcher {
    /// Handles idle responses for a given resource and task code mapping
    async fn handle_idle_response(&self, response: &str, manager_channel_map: &ManagerChannelMap) {
        let emg_response_mapping = vec![
            (
                "OPEN HAND".to_string(),
                "OPEN_FIST".to_string(),
                sgcp::Resource::Maestro,
            ),
            (
                "CLOSE HAND".to_string(),
                "CLOSE_FIST".to_string(),
                sgcp::Resource::Maestro,
            ),
        ];

        if let Some((task_code, resource)) = emg_response_mapping
            .iter()
            .find(|(resp, _, _)| resp == response)
            .map(|(_, task_code, resource)| (task_code.clone(), *resource))
        {
            let request = sgcp::Request {
                resource: resource as i32,
                task_code,
                task_data: None,
            };

            // dispatch request to maestro to execute action based on EMG response
            match dispatch_task(request, manager_channel_map).await {
                Ok(res) => info!("Task succeeded: {:?}", res),
                Err(e) => error!("Task failed: {:?}", e),
            }
        } else {
            error!("Unexpected response: {}", response);
        }
    }

    /// Processes idle tasks for a given resource
    pub async fn process_idle_task(&mut self, manager_channel_map: &ManagerChannelMap) {
        let request = sgcp::Request {
            resource: sgcp::Resource::Emg as i32,
            task_code: "IDLE".to_string(),
            task_data: None,
        };
        // dispatch request to EMG to process idle tasks
        match dispatch_task(request, manager_channel_map).await {
            Ok(res) => {
                self.handle_idle_response(res.as_str(), manager_channel_map)
                    .await
            },
            Err(err) => {
                error!("An error occurred when dispatching task; error={err}");
                log::error!("Failed to dispatch maintenance task: {:?}", err);
            },
        }
    }
}
