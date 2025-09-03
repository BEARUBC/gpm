//! Mocks the ADC reader and produces (somewhat realistic) EMG data

use super::EmgData;
use crate::resources::Resource;
use chrono::Utc;
use gpm::sgcp;
use rand::Rng;

pub struct Emg;

impl Resource for Emg {
    fn init() -> Self {
        Emg {}
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl Emg {
    // MOCK -- FOR EMG VISUALIZATION
    // TODO:Define a trait to provide a uniform interface to provide EMG (ADC) data to the EMG
    // exporter
    pub fn read_adc() -> EmgData {
        let mut rng = rand::rng();
        EmgData {
            channel_0: rng.random_range(-1.0..1.0),
            channel_1: rng.random_range(-0.5..0.5),
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}
