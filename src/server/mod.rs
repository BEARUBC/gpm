pub mod connection;
mod macros;

use crate::ManagerChannelMap;
use crate::config::Config;
use crate::managers::ManagerChannelData;
use crate::retry;
use crate::sgcp;
use anyhow::Error;
use anyhow::Result;
use connection::Connection;
use log::error;
use log::info;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::sync::oneshot;
use tokio::time::{interval, Duration};


/// Starts the main TCP listener loop -- can handle at most MAX_CONCURRENT_CONNECTIONS connections
/// at any given time
pub async fn run_server_loop(manager_channel_map: ManagerChannelMap) {
    let server_config = Config::global()
        .server
        .as_ref()
        .expect("Expected server config to be defined");

    let listener = TcpListener::bind(server_config.address.clone())
        .await
        .expect(&format!(
            "Couldn't bind to address {}",
            &server_config.address
        ));

    let sem = Arc::new(Semaphore::new(
        server_config.max_concurrent_connections as usize,
    ));
    info!("GPM server listening on {:?}", server_config.address);

    loop {
        let sem_clone = Arc::clone(&sem);
        let (stream, client_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                error!(
                    "Encountered an error when accepting new connection; error={:?}",
                    err
                );
                continue;
            },
        };

        // Stores a mapping between the manager tasks and the Sender channel needed to communicate
        // with them
        let send_channel_map = manager_channel_map.clone();
        tokio::spawn(async move {
            // Bounds number of concurrent connections
            if let Ok(_) = retry!(sem_clone.try_acquire()) {
                info!("Accpeted new remote connection from host={:?}", client_addr);
                handle_connection(stream, &send_channel_map).await.unwrap();
            } else {
                error!(
                    "Rejected new remote connection from host={:?}, currently serving maximum_clients={:?}",
                    client_addr, server_config.max_concurrent_connections
                );
            }
        });
    }
}

/// Reads protobufs from the underlying stream and dispatches tasks to the appropriate
/// resource manager
async fn handle_connection(stream: TcpStream, map: &ManagerChannelMap) -> Result<()> {
    let mut conn = Connection::new(stream);
    loop {
        match conn.read_frame().await {
            Ok(val) => match val {
                Some(req) => {
                    // Successfully read a frame
                    info!("Recieved request: {:?}", req);
                    let res = match dispatch_task(req, &map).await {
                        Ok(res) => res,
                        Err(err) => {
                            error!("An error occurred when dispatching task; error={:?}", err);
                            format!("An error occurred; Error={:?}", err)
                        },
                    };

                    match retry!(conn.write(res.as_bytes()).await) {
                        Ok(_) => (),
                        Err(err) => error!(
                            "An error occurred when writing response to peer; error={:?}",
                            err
                        ),
                    };
                },
                None => {
                    // Connection was closed cleanly
                    info!("Connection closed with peer");
                    break;
                },
            },

            Err(err) => {
                error!("Reading frame from stream failed with error={:?}", err);
                break;
            },
        }
    }
    Ok(())
}

/// Dispatches a request to the appropiate resource manager. Returns the response from the task.
async fn dispatch_task(
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

// nicks BS VV

// move this somewhere else
// event-driven task generation
// This function is responsible for monitoring events and generating requests
// based on those events. It uses a timer to trigger periodic tasks and
// simulates an event-driven architecture.
async fn handle_emg_idle_response(response: &str, manager_channel_map: &ManagerChannelMap) {
    let (task_code, resource) = match response {
        "OPEN HAND" => ("OPEN_FIST", sgcp::Resource::Maestro),
        "CLOSE HAND" => ("CLOSE_FIST", sgcp::Resource::Maestro),
        _ => {
            error!("Unexpected response: {}", response);
            return;
        }
    };

    let request = sgcp::Request {
        resource: resource as i32,
        task_code: task_code.to_string(),
        task_data: None,
    };

    match dispatch_task(request, manager_channel_map).await {
        Ok(res) => info!("Task succeeded: {:?}", res), // need to return a string here
        Err(e) => error!("Task failed: {:?}", e),
    }
}

async fn process_emg_idle_task(manager_channel_map: &ManagerChannelMap) {
    let request = sgcp::Request {
        resource: sgcp::Resource::Emg as i32,
        task_code: "IDLE".to_string(),
        task_data: None,
    };

    match dispatch_task(request, manager_channel_map).await {
        Ok(res) => handle_emg_idle_response(res.as_str(), manager_channel_map).await,
        Err(err) => {
            error!("An error occurred when dispatching task; error={err}");
            log::error!("Failed to dispatch maintenance task: {:?}", err);
        }
    }
}

// idle tasks
pub async fn monitor_events(manager_channel_map: ManagerChannelMap) {

    // init
    init_tasks(manager_channel_map.clone()).await;

    //let mut EMG_idle = interval(Duration::from_millis(2)); // 500 Hz sampling rate
    let mut EMG_idle = interval(Duration::from_millis(1000)); // 1 Hz sampling rate
    let send_channel_map = manager_channel_map.clone();
    loop {
        tokio::select! {
            _ = EMG_idle.tick() => {
                process_emg_idle_task(&send_channel_map).await;
            }
        }
    }
}

pub async fn init_tasks(manager_channel_map: ManagerChannelMap){
    // run initialization tasks
    let init_request = sgcp::Request {
        resource: sgcp::Resource::Emg as i32,
        task_code: "CALIBRATE".to_string(),
        task_data: None,
    };
    let init_map = manager_channel_map.clone();

    dispatch_task(init_request, &init_map).await;
}
