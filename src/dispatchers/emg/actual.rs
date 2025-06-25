use crate::ManagerChannelMap;

// TODO: refactor

// idle tasks
pub async fn run_emg_monitor_loop(manager_channel_map: ManagerChannelMap) {
    // init
    init_tasks(manager_channel_map.clone()).await;

    let emg_config = Config::global()
        .emg_sensor
        .as_ref()
        .expect("Expected EMG config to be defined");

    let mut emg_idle = interval(Duration::from_millis(emg_config.sampling_speed_ms)); // 1000 ms for 1 Hz sampling rate for idle tasks, 2 ms for 500 Hz sampling rate
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
    // let mut HAPTICS_idle = interval(Duration::from_millis(1000)); // 1 Hz sampling rate // example for haptics
    let send_channel_map = manager_channel_map.clone();
    loop {
        tokio::select! {
            _ = emg_idle.tick() => {
                process_idle_task(&send_channel_map, sgcp::Resource::Emg, "IDLE", &emg_response_mapping).await;
            }
            // _ = HAPTICS_idle.tick() => {
            //     // handle haptics idle task here
            // }
        }
    }
}

/// Handles idle responses for a given resource and task code mapping
async fn handle_idle_response(
    response: &str,
    manager_channel_map: &ManagerChannelMap,
    response_mapping: &[(String, String, sgcp::Resource)],
) {
    if let Some((task_code, resource)) = response_mapping
        .iter()
        .find(|(resp, _, _)| resp == response)
        .map(|(_, task_code, resource)| (task_code.clone(), *resource))
    {
        let request = sgcp::Request {
            resource: resource as i32,
            task_code,
            task_data: None,
        };

        match dispatch_task(request, manager_channel_map).await {
            Ok(res) => info!("Task succeeded: {:?}", res),
            Err(e) => error!("Task failed: {:?}", e),
        }
    } else {
        error!("Unexpected response: {}", response);
    }
}

/// Processes idle tasks for a given resource
async fn process_idle_task(
    manager_channel_map: &ManagerChannelMap,
    resource: sgcp::Resource,
    task_code: &str,
    response_mapping: &[(String, String, sgcp::Resource)],
) {
    let request = sgcp::Request {
        resource: resource as i32,
        task_code: task_code.to_string(),
        task_data: None,
    };

    match dispatch_task(request, manager_channel_map).await {
        Ok(res) => handle_idle_response(res.as_str(), manager_channel_map, response_mapping).await,
        Err(err) => {
            error!("An error occurred when dispatching task; error={err}");
            log::error!("Failed to dispatch maintenance task: {:?}", err);
        },
    }
}

async fn init_tasks(manager_channel_map: ManagerChannelMap) {
    // run initialization tasks
    let init_request = sgcp::Request {
        resource: sgcp::Resource::Emg as i32,
        task_code: "CALIBRATE".to_string(),
        task_data: None,
    };

    // can also add maestro init, move all motors to home position, 0
    let init_map = manager_channel_map.clone();

    match dispatch_task(init_request, &init_map).await {
        Ok(_) => info!("Initialization sucess"),
        Err(err) => error!("Initialization failed: {:?}", err),
    }
}
