use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Error;
use anyhow::Result;
use log::error;
use log::info;
use log::warn;
use tokio::time::{interval, Duration};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc;

use crate::config::GPM_TCP_ADDR;
use crate::config::MAX_CONCURRENT_CONNECTIONS;
use crate::connection::Connection;
use crate::import_sgcp;
use crate::managers::ManagerChannelData;
use crate::retry;
use crate::sgcp::*;
use crate::ManagerChannelMap;
use crate::Resource;
use crate::Request;
use crate::sgcp::request::TaskData;

/// Provides the boilerplate to setup routing required to send tasks to the appropriate
/// resource manager // keep for internal
macro_rules! dispatch_task {
    {$request:ident, $($variant:pat => $channel:expr),*} => {{
        let resource_key = $request.resource().as_str_name();
        match $request.resource() {
            $($variant => {
                info!("Dispatching {:?} task with task_code={:?}", resource_key, $request.task_code);
                match $channel {
                    Some(tx) => {
                        // Set up channel on which manager will send its response
                        let (resp_tx, resp_rx) = oneshot::channel::<String>();
                        tx.send(ManagerChannelData {
                            task_code: $request.task_code.as_str().to_string(),
                            task_data: $request.task_data,
                            resp_tx
                        }).await.unwrap();
                        let res = resp_rx.await.unwrap();
                        info!("{:?} task returned value={:?}", resource_key, res);
                        Ok(res)
                    },
                    None => {
                        Err(Error::msg("{resource_key} resource manager not initialized"))
                    }
                }
            }),*,
            _ => {
                Err(Error::msg("Unmatched task"))
            }
        }
    }}
}

/// Starts the main TCP listener loop -- can handle at most MAX_CONCURRENT_CONNECTIONS connections
/// at any given time // tcp - dont need
pub async fn init(manager_channel_map: ManagerChannelMap) {
    let listener = TcpListener::bind(GPM_TCP_ADDR).await.unwrap();
    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));
    info!("GPM server listening on {:?}", GPM_TCP_ADDR);
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
                error!("Rejected new remote connection from host={:?}, currently serving maximum_clients={:?}", client_addr, MAX_CONCURRENT_CONNECTIONS);
            }
        });
    }
}

/// Reads protobufs from the underlying stream and dispatches tasks to the appropriate
/// resource manager // reads from tcpstream
async fn handle_connection(mut stream: TcpStream, map: &ManagerChannelMap) -> Result<()> {
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
                            error!("An error occurred when dispatching task; error={err}");
                            conn.write("An error occurred; Error={err}".as_bytes());
                            continue;
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

/// Dispatches a request to the appropiate resource manager. Returns the response from the task. // keep for internal
pub async fn dispatch_task(
    request: Request,
    manager_channel_map: &ManagerChannelMap,
) -> Result<String> {
    dispatch_task!(
        request,
        Resource::Bms => manager_channel_map.get(Resource::Bms.as_str_name()),
        Resource::Emg => manager_channel_map.get(Resource::Emg.as_str_name()),
        Resource::Maestro => manager_channel_map.get(Resource::Maestro.as_str_name())
    )
}

/// Starts the main internal task dispatch loop for the bionic arm system.
/// Periodically sends tasks to resource managers without relying on TCP.
pub async fn cli_input(manager_channel_map: ManagerChannelMap, mut request_rx: mpsc::Receiver<Request>) {
    use tokio::time::{sleep, Duration};
    use std::io::{self, Write};

    info!("Starting internal task dispatch loop...");

    // Open channel for CLI commands
    loop {
        // Prompt user for input
        print!("Enter command (format: <RESOURCE> <TASK_CODE> [TASK_DATA]): ");
        io::stdout().flush().unwrap();

        // Read input from the user
        let mut input = String::new();
        if let Err(err) = io::stdin().read_line(&mut input) {
            error!("Failed to read input: {:?}", err);
            continue;
        }

        // Parse the input into components
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.len() < 2 {
            error!("Invalid command format. Expected: <RESOURCE> <TASK_CODE> [TASK_DATA]");
            continue;
        }

        // Extract resource, task code, and optional task data
        let resource = match parts[0].to_uppercase().as_str() {
            "BMS" => Resource::Bms as i32,
            "EMG" => Resource::Emg as i32,
            "MAESTRO" => Resource::Maestro as i32,
            _ => {
                error!("Unknown resource: {}", parts[0]);
                continue;
            }
        };

        let task_code = parts[1].to_string();
        let task_data = if parts.len() > 2 {
            match resource {
            x if x == Resource::Bms as i32 => {
                None // Add appropriate handling for Bms if needed
            }
            x if x == Resource::Emg as i32 => {
                None // Add appropriate handling for Emg if needed
            }
            x if x == Resource::Maestro as i32 => {
                let targets: Vec<i32> = parts[2..]
                    .iter()
                    .filter_map(|s| s.parse::<i32>().ok())
                    .collect();
                let channels: Vec<i32> = vec![]; // Provide appropriate channel data if needed
                Some(TaskData::MaestroData(super::maestro::TaskData {
                    targets,
                    channels,
                }))
            }
            _ => {
                error!("Unsupported resource type for task data");
                None
            }
            }
        } else {
            None
        };

        // Create the request
        let request = Request {
            resource,
            task_code,
            task_data,
        };

        // Pass the request to handle_task
        match handle_task(request, &manager_channel_map).await {
            Ok(response) => info!("Task succeeded: {:?}", response),
            Err(e) => error!("Task failed: {:?}", e),
        }
    }
}


pub async fn handle_task(request: Request, map: &ManagerChannelMap) -> Result<String> {
    dispatch_task!(
        request,
        Resource::Bms => map.get(Resource::Bms.as_str_name()),
        Resource::Emg => map.get(Resource::Emg.as_str_name()),
        Resource::Maestro => map.get(Resource::Maestro.as_str_name())
    )
}

// event-driven task generation
// This function is responsible for monitoring events and generating requests
// based on those events. It uses a timer to trigger periodic tasks and
// simulates an event-driven architecture.
async fn handle_emg_idle_response(response: &str, manager_channel_map: &ManagerChannelMap) {
    let (task_code, resource) = match response {
        "OPEN HAND" => ("OPEN_FIST", Resource::Maestro),
        "CLOSE HAND" => ("CLOSE_FIST", Resource::Maestro),
        _ => {
            error!("Unexpected response: {}", response);
            return;
        }
    };

    let request = Request {
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
    let request = Request {
        resource: Resource::Emg as i32,
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
    let mut EMG_idle = interval(Duration::from_millis(2)); // 500 Hz sampling rate

    loop {
        tokio::select! {
            _ = EMG_idle.tick() => {
                process_emg_idle_task(&manager_channel_map).await;
            }
        }
    }
}

// implement command line interface to manually trigger tasks

