pub mod http;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPoint {
    timestamp: DateTime<Utc>,
    value: f32,
}
