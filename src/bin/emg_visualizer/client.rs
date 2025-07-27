//! TCP Client for the GPM EMG exporter

use crate::{AppResult, DataPoint, SERVER_ADDR};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
    sync::mpsc,
};

pub struct EmgExporterClient {
    tx: mpsc::Sender<DataPoint>,
}

impl EmgExporterClient {
    pub fn new(tx: mpsc::Sender<DataPoint>) -> Self {
        EmgExporterClient { tx }
    }

    /// The "producer" task: connects to the server and sends data points into the channel.
    pub async fn listen_for_data(&self) -> AppResult<()> {
        let stream = TcpStream::connect(SERVER_ADDR).await?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        while reader.read_line(&mut line).await? > 0 {
            let point: DataPoint = serde_json::from_str(&line)?;
            // Send data to the UI task. If the receiver is dropped, this will fail.
            if self.tx.send(point).await.is_err() {
                break;
            }
            line.clear();
        }
        Ok(())
    }
}
