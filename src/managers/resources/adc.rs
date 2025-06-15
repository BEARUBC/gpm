extern crate rppal;

use anyhow::Result;
use anyhow::Error;
use log::*;

use rppal::gpio::{OutputPin};
use rppal::spi::{Spi};
use std::{io, thread, time::Duration};

pub fn process_data(values: Vec<u16>, inner_threshold: u16, outer_threshold: u16) -> Result<i32> {
    if values.len() != 2 {
        return Err(Error::msg("Expected 2 EMG values"));
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

fn average(list: Vec<u16>) -> Result<u16> {
    if list.is_empty() {
        return Err(Error::msg("Cannot calculate average of an empty list"));
    } else {
        let sum: u32 = list.iter().map(|&x| x as u32).sum();
        Ok((sum / list.len() as u32) as u16)
    }
}
    
pub fn calibrate_emg(buffer_size : usize, spi: &mut Spi, cs: &mut OutputPin, pause_duration : u64) -> Result<[u16; 2]> {
    let inner_buffer = read_samples(0, cs, buffer_size, spi, "inner", pause_duration);

    info!("\nFinished inner sampling. Press ENTER when you're ready to start outer sampling...");

    let _ = io::stdin().read_line(&mut String::new());

    let outer_buffer = read_samples(1, cs, buffer_size, spi, "outer", pause_duration);

    let avg_inner = average(inner_buffer.clone()).unwrap_or_else(|e| {
        info!("Error calculating average for inner buffer: {}", e);
        0
    });

    let avg_outer = average(outer_buffer.clone()).unwrap_or_else(|e| {
        info!("Error calculating average for outer buffer: {}", e);
        0
    });

    Ok([avg_inner, avg_outer])
}

pub fn read_samples(channel: u8, cs: &mut OutputPin, sample_count: usize, spi: &mut Spi, label: &str, pause_duration : u64) -> Vec<u16> {
    let mut buffer = Vec::with_capacity(sample_count);
    info!("Flex {label}");

    while buffer.len() < sample_count {
        match read_adc_channel(channel, cs, spi) {
            Ok(value) => buffer.push(value),
            Err(_) => info!("Error reading SPI on channel {channel} during {label}"),
        }
        thread::sleep(Duration::from_millis(pause_duration));
    }

    buffer
}


pub fn read_adc_channels(channels: &[u8], cs: &mut OutputPin, spi: &mut Spi) -> Result<Vec<u16>> {
    channels
        .iter()
        .map(|&channel| read_adc_channel(channel, cs, spi))
        .collect::<Result<Vec<u16>, anyhow::Error>>()
}

pub fn read_adc_channel(channel: u8, cs: &mut OutputPin, spi: &mut Spi) -> Result<u16> {
    if channel > 7 {
        return Err(Error::msg("Invalid channel. Must be between 0 and 7."));
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