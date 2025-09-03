//! Tiny EMG Data TCP Exporter

use anyhow::{Context, Result, anyhow};
use log::{error, info, warn};
use serde_json::to_string;
use std::{collections::HashMap, time::Duration};
use crate::{config::CommandDispatchStrategy, managers::ManagerChannelData};
use crate::dispatchers::dispatch_task;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
    time::interval,
};
use tokio::sync::mpsc::Sender;
use crate::{config::Config, resources::emg::Emg};
use gpm::sgcp;

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
            tokio::spawn(handle_connection(stream, self.interval_duration, self.manager_channel_map.clone()));
        }
    }
}

/// Handles an individual client connection.
async fn handle_connection(stream: TcpStream, interval_duration: Duration, manager_channel_map: ManagerChannelMap) {
    let mut writer = BufWriter::new(stream);
    let mut interval = interval(interval_duration);
    
    loop {
        interval.tick().await;
        
        let result = match Config::global().command_dispatch_strategy {
            CommandDispatchStrategy::Tcp => get_emg_json_direct().await, // Option A: direct call
            _ => get_emg_json_dispatched(&manager_channel_map).await,    // Option B: dispatched task
        };

        let json_string = match result {
            Ok(s) => s,
            Err(e) => {
                warn!("Skipping EMG tick due to error: {}", e);
                continue;
            }
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

/// Dispatcher mode: send request through the resource manager
pub async fn get_emg_json_dispatched(manager_channel_map: &ManagerChannelMap) -> Result<String> {
    let request = sgcp::Request {
        resource: sgcp::Resource::Emg as i32,
        task_code: "EXPORT".to_string(),
        task_data: None,
    };

    match dispatch_task(request, manager_channel_map).await {
        Ok(response) => {
            info!("(Dispatched) Got EMG ADC Data JSON: {}", response);
            Ok(response)
        }
        Err(e) => {
            warn!("(Dispatched) Failed to dispatch EMG request: {}", e);
            Err(anyhow!("Dispatch failed: {}", e))
        }
    }
}


/// Direct mode: call `Emg::read_adc()` directly
pub async fn get_emg_json_direct() -> Result<String> {
    let emg_data = Emg::read_adc();

    match to_string(&emg_data) {
        Ok(json) => {
            info!("(Direct) EMG ADC Data JSON: {}", json);
            Ok(json)
        }
        Err(e) => {
            warn!("(Direct) Failed to serialize EMG data: {}", e);
            Err(anyhow!("Serialization failed: {}", e))
        }
    }
}