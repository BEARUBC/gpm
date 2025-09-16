//! Tiny EMG Data TCP Exporter

use anyhow::{Context, Result};
use log::{error, info, warn};
use serde_json::to_string;
use std::{sync::Mutex, time::Duration};
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
    time::interval,
};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use crate::{config::Config, resources::emg::Emg, managers::Manager};

pub struct Exporter {
    address: String,
    interval_duration: Duration,
    manager: Arc<TokioMutex<Manager<Emg>>>,
}

impl Exporter {
    pub fn new(emg_manager: Arc<TokioMutex<Manager<Emg>>>) -> Self {
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
            manager: emg_manager,
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

            let manager_clone = self.manager.clone(); // clone Arc
            let interval_duration = self.interval_duration;

            tokio::spawn(handle_connection(manager_clone, stream, interval_duration));
        }
    }
}

/// Handles an individual client connection.
async fn handle_connection(manager: Arc<TokioMutex<Manager<Emg>>>, stream: TcpStream, interval_duration: Duration) {
    let mut writer = BufWriter::new(stream);
    let mut interval = interval(interval_duration);

    loop {
        interval.tick().await;

        let emg_data = {
            let manager = manager.lock().await;
            manager.get_resource().read_adc()
        };

        let json_string = match to_string(&emg_data) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to serialize EMG data: {}", e);
                continue;
            },
        };

        let payload = format!("{}\n", json_string);

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
