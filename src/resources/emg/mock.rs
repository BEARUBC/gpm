//! Mocks the ADC reader and produces (somewhat realistic) EMG data

use crate::resources::Resource;
use gpm::sgcp;

use super::AdcReader;

pub struct Emg;

impl Resource for Emg {
    fn init() -> Self {
        Emg {}
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl AdcReader for Emg {
    // TODO: @kumarpit implement mocking
    fn read_adc(&self, _channel: u8, _label: &str) -> Vec<u16> {
        return vec![1, 2, 3, 4, 5, 6];
    }
}
