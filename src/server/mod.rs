use std::sync::Arc;

use anyhow::Result;
use log::error;
use log::info;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::Semaphore;

use crate::config::GPM_TCP_ADDR;
use crate::config::MAX_TCP_CONNECTIONS;
use crate::ManagerChannelMap;
use crate::_dispatch_task as dispatch_task;
use crate::import_sgcp;
use crate::managers::ManagerChannelData;
use crate::sgcp::*;
use crate::streaming::Connection;

pub async fn init_gpm_listener(manager_channel_map: ManagerChannelMap) {
    let listener = TcpListener::bind(GPM_TCP_ADDR).await.unwrap();
    let sem = Arc::new(Semaphore::new(MAX_TCP_CONNECTIONS));
    info!("Listening on {:?}", GPM_TCP_ADDR);
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
        let send_channel_map = manager_channel_map.clone();
        tokio::spawn(async move {
            if let Ok(_) = sem_clone.try_acquire() {
                // Bounds number of concurrent connections
                info!("Accpeted new remote connection from host={:?}", client_addr);
                handle_connection(stream, &send_channel_map).await.unwrap();
            } else {
                error!("Rejected new remote connection from host={:?}, currently serving maximum_clients={:?}", client_addr, MAX_TCP_CONNECTIONS)
            }
        });
    }
}

/// Parses protobuf struct from stream and handles the request.
async fn handle_connection(mut stream: TcpStream, map: &ManagerChannelMap) -> Result<()> {
    let mut conn = Connection::new(stream);
    loop {
        match conn.read_frame().await.unwrap() {
            Some(req) => {
                info!("Recieved request: {:?}", req);
                let res = dispatch_task(req, &map).await.unwrap();
                conn.write(res.as_bytes()).await;
            },
            _ => todo!(),
        }
    }
    Ok(())
}

dispatch_task! {
    Component::Bms => (bms::Task, request::TaskData::BmsData, ManagerChannelData::BmsChannelData),
    Component::Emg => (emg::Task, request::TaskData::EmgData, ManagerChannelData::EmgChannelData),
    Component::Maestro => (maestro::Task, request::TaskData::MaestroData, ManagerChannelData::MaestroChannelData)
}
