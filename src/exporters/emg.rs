//! Tiny EMG Data TCP Exporter

use anyhow::{Context, Result};
use log::{error, info, warn};
use serde_json::to_string;
use std::time::Duration;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
    time::interval,
};

use crate::{config::Config, resources::emg::Emg};
type ManagerChannelMap = HashMap<String, Sender<ManagerChannelData>>;
pub struct Exporter {
    address: String,
    interval_duration: Duration,
    manager_channel_map: ManagerChannelMap,
}

impl Exporter {
    pub fn new(manager_channel_map: ManagerChannelMap) -> Self {
        // ew
        let emg_telemetry_config = Config::global()
            .telemetry
            .as_ref()
            .unwrap()
            .emg
            .as_ref()
            .unwrap();

        Exporter {
            address: emg_telemetry_config.address.clone(),
            interval_duration: Duration::from_millis(emg_telemetry_config.tick_interval_in_millis),
            manager_channel_map
        }
    }

    /// Main server loop
    pub async fn init(self) -> Result<()> {
        let listener = TcpListener::bind(&self.address)
            .await
            .with_context(|| format!("Failed to bind TCP listener to {}", self.address))?;

        info!("EMG exporter listening on {}", self.address);

        loop {
            let (stream, client_addr) = match listener.accept().await {
                Ok(connection) => connection,
                Err(e) => {
                    error!("Failed to accept connection: {:?}", e);
                    continue;
                },
            };

            info!("Accepted new connection from: {}", client_addr);
            tokio::spawn(handle_connection(stream, self.interval_duration, self.manager_channel_map));
        }
    }
}

/// Handles an individual client connection.
async fn handle_connection(stream: TcpStream, interval_duration: Duration, manager_channel_map: ManagerChannelMap) {
    let mut writer = BufWriter::new(stream);
    let mut interval = interval(interval_duration);

    loop {
        interval.tick().await;

        let request = sgcp::Request {
            resource: sgcp::Resource::Emg as i32,
            task_code: "EXPORT".to_string(),
            task_data: None,
        };

        let response = match dispatch_task(request, &manager_channel_map).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to dispatch EMG request: {}", e);
                continue;
            }
        };

        let payload = format!("{}\n", response);

        if let Err(e) = writer.write_all(payload.as_bytes()).await {
            info!("Connection closed during write: {}", e);
            break;
        }

        if let Err(e) = writer.flush().await {
            info!("Connection closed during flush: {}", e);
            break;
        }
    }
    info!("Client disconnected.");
}
