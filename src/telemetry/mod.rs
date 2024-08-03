pub mod http;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct MetricDataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
}
