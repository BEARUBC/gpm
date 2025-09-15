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

pub struct Exporter {
    address: String,
    interval_duration: Duration,
}

impl Exporter {
    pub fn new() -> Self {
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
            tokio::spawn(handle_connection(stream, self.interval_duration));
        }
    }
}

/// Handles an individual client connection.
async fn handle_connection(stream: TcpStream, interval_duration: Duration) {
    let mut writer = BufWriter::new(stream);
    let mut interval = interval(interval_duration);

    loop {
        interval.tick().await;

        let json_string = "test"; // Replaced due to bugs during emg testing, will put back og implementation after

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
