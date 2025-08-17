use anyhow::Context;
use anyhow::Result;
use gpm::sgcp;

use crate::config::Config;
use crate::resources::Resource;
use crate::resources::common::Adc;

pub struct Fsr {
    pub at_rest_threshold: u16,
    pub pressure_threshold: u16,
    pub clock_speed: u32,
    pub num_fsrs: usize,
    pub cs_pins: Vec<u8>,
    pub num_channels: u8,
}

impl Resource for Fsr {
    fn init() -> Self {
        let fsr_config = Config::global()
            .dispatcher
            .fsr
            .as_ref()
            .expect("Expected emg config to be defined");
        let fsr = Fsr {
            at_rest_threshold: fsr_config.at_rest_threshold,
            pressure_threshold: fsr_config.pressure_threshold,
            clock_speed: fsr_config.clock_speed,
            num_fsrs: fsr_config.num_fsrs,
            cs_pins: fsr_config.cs_pins.clone(),
            num_channels: 8,
        };
        // checks
        if fsr.num_fsrs != fsr.cs_pins.len() {
            panic!("Number of expected FSRs did not match the number of CS Pins");
        }

        fsr
    }

    fn name() -> String {
        sgcp::Resource::Fsr.as_str_name().to_string()
    }
}

impl Fsr {
    // TODO: refine this function later on.
    pub fn process_data(&mut self) -> Result<u8> {
        const VIBRATE_OFF: u8 = 0;
        const VIBRATE_ON: u8 = 1;
        let mut result: u8 = 0;

        // serialized now but should be concurrent readings, maybe?
        for cs_pin in self.cs_pins.clone() {
            let mut adc = Adc::init(cs_pin, self.clock_speed);
            for channel in 0..self.num_channels {
                let value = adc
                    .read_channel(channel)
                    .with_context(|| format!("Failed to read from ADC channel {}", channel))?;


                // if at least one of the channels crossed the threshold, turn on vibrate
                if value < self.at_rest_threshold {
                    result |= VIBRATE_ON;
                } else {
                    result |= VIBRATE_OFF;
                }
            }
        }

        Ok(result)
    }
}
