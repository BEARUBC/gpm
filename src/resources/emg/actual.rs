// All tasks operating on the EMG system live in this file
use crate::config::Config;
use crate::resources::common::Adc;
use anyhow::{Error, Result};
use log::*;

use crate::resources::Resource;
use crate::sgcp;
use rppal::gpio::OutputPin;
use rppal::spi::Spi;
use std::{io, thread, time::Duration};

use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub struct Emg {
    pub adc: Adc,
    pub buffer_size: usize,
    pub inner_threshold: u16,
    pub outer_threshold: u16,
    pub inter_channel_sample_duration: u64, // different from sampling speed, this is the time between reading the inner and outer channels
}

impl Resource for Emg {
    fn init() -> Self {
        let emg_config = Config::global()
            .dispatcher
            .emg
            .as_ref()
            .expect("Expected emg config to be defined");

        let adc = Adc::init(emg_config.cs_pin);

        Emg {
            adc,
            buffer_size: emg_config.buffer_size,
            inner_threshold: 0,
            outer_threshold: 0,
            inter_channel_sample_duration: emg_config.pause_duration_ms,
        }
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl Emg {
    pub fn process_data(&self, values: Vec<u16>) -> Result<i32> {
        if values.len() != 2 {
            return Err(Error::msg("Expected 2 EMG values"));
        }

        if values[0] >= self.inner_threshold && values[1] <= self.outer_threshold {
            Ok(1) // Open
        } else if values[0] <= self.inner_threshold && values[1] >= self.outer_threshold {
            Ok(0) // Close
        } else {
            Ok(-1) // No action
        }
    }

    pub fn calibrate_emg(&mut self) -> Result<()> {
        let inner_buffer = self.read_samples(0, "inner");

        info!(
            "\nFinished inner sampling. Press ENTER when you're ready to start outer sampling..."
        );

        let _ = io::stdin().read_line(&mut String::new());

        let outer_buffer = self.read_samples(1, "outer");

        let avg_inner = Adc::average_values(inner_buffer.as_ref()).unwrap_or_else(|e| {
            info!("Error calculating average for inner buffer: {}", e);
            0
        });

        let avg_outer = Adc::average_values(outer_buffer.as_ref()).unwrap_or_else(|e| {
            info!("Error calculating average for outer buffer: {}", e);
            0
        });

        self.inner_threshold = avg_inner;
        self.outer_threshold = avg_outer;

        Ok(())
    }

    pub fn read_samples(&mut self, channel: u8, label: &str) -> Vec<u16> {
        let mut buffer = Vec::with_capacity(self.buffer_size);
        info!("Flex {label}");

        while buffer.len() < self.buffer_size {
            match self.adc.read_channel(channel) {
                Ok(value) => buffer.push(value),
                Err(_) => info!("Error reading SPI on channel {channel} during {label}"),
            }
            thread::sleep(Duration::from_millis(self.inter_channel_sample_duration));
        }

        buffer
    }

    pub fn read_adc_channels(&mut self, channels: &[u8]) -> Result<Vec<u16>> {
        self.adc.read_channels(channels)
    }
}
