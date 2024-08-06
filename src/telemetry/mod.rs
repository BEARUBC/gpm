pub mod http;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

/// Represents a timestamped data point to feed into Grafana
#[derive(Serialize, Deserialize, Debug)]
pub struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
}
