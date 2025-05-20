pub mod connection;

use std::sync::Arc;

use anyhow::Error;
use anyhow::Result;
use log::error;
use log::info;
use log::warn;
use request::TaskData::*;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::sync::oneshot;

use crate::ManagerChannelMap;
use crate::config::Config;
use crate::import_sgcp;
use crate::managers::ManagerChannelData;
use crate::retry;
use crate::sgcp::*;
use connection::Connection;

/// Provides the boilerplate to setup routing required to send tasks to the appropriate
/// resource manager
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
                        })
                        .await
                        .map_err(|e| Error::msg(format!("Failed to send command to {} manager: {:?}", resource_key, e)))?;

                        let res = resp_rx
                            .await
                            .map_err(|e| Error::msg(format!("Failed to read response from {} manager: {:?}", resource_key, e)))?;

                        info!("{} task returned value={:?}", resource_key, res);

                        Ok(res)
                    },
                    None => {
                        Err(Error::msg(format!("{} resource manager not initialized", resource_key)))
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
