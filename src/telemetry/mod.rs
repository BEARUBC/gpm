pub mod http;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
}