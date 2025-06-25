// All tasks operating on the EMG system live in this file
use crate::config::Config;
use anyhow::{Context, Error, Result};
use log::*;

use crate::resources::Resource;
use crate::sgcp;
use rppal::gpio::OutputPin;
use rppal::spi::Spi;
use std::{io, thread, time::Duration};

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

        let avg_inner = Self::average(inner_buffer.as_ref()).unwrap_or_else(|e| {
            info!("Error calculating average for inner buffer: {}", e);
            0
        });

        let avg_outer = Self::average(outer_buffer.as_ref()).unwrap_or_else(|e| {
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
            match self.read_adc_channel(channel) {
                Ok(value) => buffer.push(value),
                Err(_) => info!("Error reading SPI on channel {channel} during {label}"),
            }
            thread::sleep(Duration::from_millis(self.inter_channel_sample_duration));
        }

        buffer
    }

    pub fn read_adc_channels(&mut self, channels: &[u8]) -> Result<Vec<u16>> {
        channels
            .iter()
            .map(|&channel| {
                self.read_adc_channel(channel)
                    .with_context(|| format!("Failed to read from ADC channel {}", channel))
            })
            .collect()
    }

    /// Reads the 10-bit ADC value from a given channel (0–7) on the MCP3008 via SPI.
    /// MCP3008 messaging protocol: 3 byte message structure
    /// doc link: https://www.mathworks.com/help/matlab/supportpkg/analog-input-using-spi.html
    pub fn read_adc_channel(&mut self, channel: u8) -> Result<u16> {
        if channel > 7 {
            return Err(Error::msg(format!(
                "Invalid ADC channel: {}. Must be between 0 and 7.",
                channel
            )));
        }

        // first byte: start sequence, sends 1 as start bit
        let start_bit = 0b00000001;

        // second byte: mode (differential or single read) and channel select
        // sets bit 7 to 1 first, then left-shifts channel number by 4 bits to put it into bits 6,5,4
        // then combines with bitwise OR operator
        // remaining bits are ignored
        let config_bits = 0b10000000 | (channel << 4);

        // third byte: dummy to clock out rest of ADC result
        let tx = [start_bit, config_bits, 0x00];

        // response buffer
        let mut rx = [0u8; 3];

        //activate chip select
        self.cs_pin.set_low();

        // full-duplex SPI transfer: sends tx[], fills rx[]
        self.spi
            .transfer(&mut rx, &tx)
            .context("SPI transfer failed during ADC read")?;

        // deactivate chip select
        self.cs_pin.set_high();

        // adc sends back 3 byte response
        // actual response is 10 bits and spread across rx[1](bits 9-8) and rx[2](bits 7-0)
        // isolate rx[1] result bits, then shift them to 9-8 in a 16 bit number, then add remaining bits by combining them with bitwise OR
        let result = ((rx[1] & 0b00000011) as u16) << 8 | (rx[2] as u16);
        Ok(result)
    }

    fn average(list: &Vec<u16>) -> Result<u16> {
        if list.is_empty() {
            return Err(Error::msg("Cannot calculate average of an empty list"));
        } else {
            let sum: u32 = list.iter().map(|&x| x as u32).sum();
            Ok((sum / list.len() as u32) as u16)
        }
    }
}
