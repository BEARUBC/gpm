use crate::config::Config;

pub struct Fsr {
    pub adc: Adc,
    pub at_rest_threshold: u64,
    pub pressure_threshold: u64,
    pub num_fsrs: u8,
    pub cs_pins: Vec<u8>,
}

impl Resource for Fsr {
    fn init() -> Self {
        Self
    }
}
