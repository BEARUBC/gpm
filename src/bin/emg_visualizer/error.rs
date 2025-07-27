//! Error Types

use thiserror::Error;
use tokio::{io, sync::mpsc};

use crate::DataPoint;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Channel send error: {0}")]
    SendError(#[from] mpsc::error::SendError<DataPoint>),
}
