pub mod connection;

use super::Dispatcher;
use super::dispatch_task;
use crate::ManagerChannelMap;
use crate::config::Config;
use anyhow::Result;
use connection::Connection;
use gpm::retry;
use log::error;
use log::info;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

pub struct TcpDispatcher;

impl Dispatcher for TcpDispatcher {
    /// Starts the main TCP listener loop -- can handle at most MAX_CONCURRENT_CONNECTIONS connections
    /// at any given time
    async fn run(manager_channel_map: ManagerChannelMap) {
        let server_config = &Config::global().dispatcher.tcp;

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
