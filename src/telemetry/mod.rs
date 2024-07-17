pub mod http;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MetricDataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
}
