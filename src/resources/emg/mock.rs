//! Mocks the ADC reader and produces (somewhat realistic) EMG data

use crate::resources::Resource;
use chrono::Utc;
use gpm::sgcp;
use rand::Rng;

pub struct Emg;

#[derive(serde::Serialize)]
pub struct EmgData {
    pub channel_0: f64,
    pub channel_1: f64,
    pub timestamp: u64,
}

impl Resource for Emg {
    fn init() -> Self {
        Emg {}
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl Emg {
    pub fn read_adc() -> EmgData {
        let mut rng = rand::rng();
        EmgData {
            channel_0: rng.random_range(-1.0..1.0),
            channel_1: rng.random_range(-0.5..0.5),
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}
