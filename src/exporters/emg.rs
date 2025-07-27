//! Tiny TCP server exposing EMG data

use crate::{config::Config, resources::emg::Emg};
use log::{error, info};
use serde_json::to_string;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    time::{Duration, interval},
};

pub struct Exporter {}

impl Exporter {
    pub fn new() -> Self {
        Exporter {}
    }

    pub async fn init(&self) {
        let telemetry_emg_config = Config::global()
            .telemetry
            .as_ref()
            .unwrap()
            .emg
            .as_ref()
            .unwrap();

        let listener = TcpListener::bind(telemetry_emg_config.address.clone())
            .await
            .unwrap();

        info!(
            "Emg exporter listening on {:?}",
            telemetry_emg_config.address
        );

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            info!("Accepted connection at EMG exporter server");
            tokio::spawn(Self::handle_client(socket));
        }
    }

    async fn handle_client(socket: TcpStream) {
        let (_reader, mut writer) = socket.into_split();
        let mut interval = interval(Duration::from_millis(50)); // Send data every 50ms

        loop {
            interval.tick().await;

            let emg_data = Emg::read_adc();
            let json_string = match to_string(&emg_data) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to serialize EMG data: {}", e);
                    continue;
                },
            };

            // Send data followed by a newline to act as a delimiter
            if let Err(e) = writer
                .write_all(format!("{}\n", json_string).as_bytes())
                .await
            {
                error!(
                    "Failed to write to socket: {}. Client likely disconnected.",
                    e
                );
                break; // Client disconnected
            }
        }
        info!("Client disconnected.");
    }
}
