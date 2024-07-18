use std::sync::Arc;
use log::{error, info};
use tokio::{net::{TcpListener, TcpStream}, sync::{oneshot, Semaphore}};
use anyhow::Result;
use crate::{config::{GPM_TCP_ADDR, MAX_TCP_CONNECTIONS}, ManagerChannelMap, _dispatch_task as dispatch_task, import_sgcp, managers::ManagerChannelData, streaming::Connection};

use crate::sgcp::*;

pub async fn init_gpm_listener(manager_channel_map: ManagerChannelMap) {
    let listener = TcpListener::bind(GPM_TCP_ADDR).await.unwrap();
    let sem = Arc::new(Semaphore::new(MAX_TCP_CONNECTIONS));
    info!("Listening on {:?}", GPM_TCP_ADDR);
    loop {
        let sem_clone = Arc::clone(&sem);
        let (stream, client_addr) = listener.accept().await.unwrap();
        let send_channel_map = manager_channel_map.clone();
        tokio::spawn(async move {
            let aq = sem_clone.try_acquire();
            if let Ok(_) = aq {
                info!("Accpeted new remote connection from host={:?}", client_addr);
                handle_connection(stream, &send_channel_map).await.unwrap();
            } else {
                error!("Rejected new remote connection from host={:?}, currently serving maximum_clients={:?}", client_addr, MAX_TCP_CONNECTIONS)
            }
        });
    }
}

// Parses protobuf struct from stream and handles the request.
async fn handle_connection(mut stream: TcpStream, map: &ManagerChannelMap) -> Result<()> {
    // @todo: krarpit implement framing abstraction for tcp stream
    let mut conn = Connection::new(stream);
    match conn.read_frame().await.unwrap() {
        Some(req) => {
            info!("Recieved request: {:?}", req);
            let res = dispatch_task(req, &map).await.unwrap();
            // stream.write(res.as_bytes()).await.unwrap();
        },
        _ => todo!()
    }
    Ok(())
}

dispatch_task! {
    Component::Bms => (bms::Task, request::TaskData::BmsData, ManagerChannelData::BmsChannelData),
    Component::Emg => (emg::Task, request::TaskData::EmgData, ManagerChannelData::EmgChannelData),
    Component::Maestro => (maestro::Task, request::TaskData::MaestroData, ManagerChannelData::MaestroChannelData)
}
