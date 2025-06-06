extern crate rppal;

use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::Resource;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::sgcp;
use crate::sgcp::emg::*;
use crate::todo;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;
use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::process::Output;
use std::{error::Error as StdError, io, io::Write, thread, time::Duration};


use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

const DEFAULT_BUFFER_SIZE: usize = 100;
const SPI_DEVICE_PATH: &str = "/dev/spidev0.0";
const PAUSE_DURATION_MS: u64 = 500; // Pause duration in milliseconds

pub fn process_data(values: Vec<u16>, inner_threshold: u16, outer_threshold: u16) -> Result<i32, Box<dyn StdError>> {
    if values.len() != 2 {
        return Err("Expected 2 EMG values".into());
    }

    if values[0] >= inner_threshold && values[1] <= outer_threshold{
        Ok(1) // Open
    } else if values[0] <= inner_threshold && values[1] >= outer_threshold {
        Ok(0) // Close
    }
    else {
        Ok(-1) // No action
    }
}

fn average(list: Vec<u16>) -> Result<u16, &'static str> {
    if list.is_empty() {
        Err("Cannot calculate average of an empty list")
    } else {
        let sum: u32 = list.iter().map(|&x| x as u32).sum();
        Ok((sum / list.len() as u32) as u16)
    }
}
    
pub fn calibrate_emg(inner_read_buffer_size: usize, outer_read_buffer_size: usize, spi: &mut Spi, cs: &mut OutputPin) -> [u16; 2] {
    let inner_buffer = read_samples(0, cs, inner_read_buffer_size, spi, "inner");

    print!("\nFinished inner sampling. Press ENTER when you're ready to start outer sampling...");
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut String::new());

    let outer_buffer = read_samples(1, cs, outer_read_buffer_size, spi, "outer");

    let avg_inner = average(inner_buffer.clone()).unwrap_or_else(|e| {
        println!("Error calculating average for inner buffer: {}", e);
        0
    });

    let avg_outer = average(outer_buffer.clone()).unwrap_or_else(|e| {
        println!("Error calculating average for outer buffer: {}", e);
        0
    });

    [avg_inner, avg_outer]
}

pub fn read_samples(channel: u8, cs: &mut OutputPin, sample_count: usize, spi: &mut Spi, label: &str) -> Vec<u16> {
    let mut buffer = Vec::with_capacity(sample_count);
    println!("Flex {label}");

    while buffer.len() < sample_count {
        match read_adc_channel(channel, cs, spi) {
            Ok(value) => buffer.push(value),
            Err(_) => println!("Error reading SPI on channel {channel} during {label}"),
        }
        thread::sleep(Duration::from_millis(PAUSE_DURATION_MS as u64));
    }

    buffer
}


pub fn read_adc_channels(channels: &[u8], cs: &mut OutputPin, spi: &mut Spi) -> Result<Vec<u16>, Box<dyn StdError>> {
    channels.iter().map(|&channel| read_adc_channel(channel, cs, spi)).collect()
}

pub fn read_adc_channel(channel: u8, cs: &mut OutputPin, spi: &mut Spi) -> Result<u16, Box<dyn StdError>> {
    if channel > 7 {
        return Err("Invalid channel. Must be between 0 and 7.".into());
    }

    let start_bit = 0b00000001;
    let config_bits = 0b10000000 | (channel << 4);

    let tx = [start_bit, config_bits, 0x00];
    let mut rx = [0u8; 3];

    cs.set_low();
    spi.transfer(&mut rx, &tx)?;
    cs.set_high();

    let result = ((rx[1] & 0b00000011) as u16) << 8 | (rx[2] as u16);
    Ok(result)
}