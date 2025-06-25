// All tasks operating on the EMG system live in this file
use crate::config::Config;
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::resources::Resource;
use crate::sgcp;
use crate::sgcp::emg::*;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;

use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub struct Emg {
    pub spi: Spi,
    pub buffer_size: usize,
    pub inner_threshold: u16,
    pub outer_threshold: u16,
    pub cs_pin: OutputPin,
    pub inter_channel_sample_duration: u64, // different from sampling speed, this is the time between reading the inner and outer channels
}

impl Resource for Emg {
    fn init() -> Self {
        let emg_config = Config::global()
            .emg_sensor
            .as_ref()
            .expect("Expected emg config to be defined");

        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 500_000, Mode::Mode0)
            .expect("Failed to initialize SPI");

        let gpio = Gpio::new().expect("Failed to initialize manual CS");

        let mut cs = gpio
            .get(emg_config.cs_pin)
            .expect("Failed to get GPIO pin for CS")
            .into_output();

        cs.set_high();

        Emg {
            spi,
            buffer_size: emg_config.buffer_size,
            inner_threshold: 0,
            outer_threshold: 0,
            cs_pin: cs,
            inter_channel_sample_duration: emg_config.pause_duration_ms,
        }
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}
