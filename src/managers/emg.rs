// All tasks operating on the EMG system live in this file
// NOTE: All EMG proccessing will now be handled on the Jetson Nano instead of GPM. Leaving
//       this file here in case we decide to implement an EMG interface for GPM anyway.
use anyhow::Error;
use anyhow::Result;
use log::*;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

use super::Manager;
use super::ManagerChannelData;
use super::Resource;
use super::ResourceManager;
use super::Responder;
use super::MAX_MPSC_CHANNEL_BUFFER;
use super::TASK_SUCCESS;
use crate::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::run;
use crate::sgcp;
use crate::sgcp::emg::*;
use crate::todo;
use crate::verify_channel_data;
use crate::managers::UNDEFINED_TASK;
use crate::managers::TASK_FAILURE;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::{error::Error as StdError, io, io::Write, thread, time::Duration};

const DEFAULT_BUFFER_SIZE: usize = 100;
const SPI_DEVICE_PATH: &str = "/dev/spidev0.0";
const PAUSE_DURATION_MS: u64 = 500; // Pause duration in milliseconds

fn average(list: Vec<u16>) -> Result<u16, &'static str> {
    if list.is_empty() {
        Err("Cannot calculate average of an empty list")
    } else {
        let sum: u32 = list.iter().map(|&x| x as u32).sum();
        Ok((sum / list.len() as u32) as u16)
    }
}


pub struct Emg {
    spi: Spi,
    inner_read_buffer_size: usize,
    outer_read_buffer_size: usize,
    inner_threshold: u16,
    outer_threshold: u16,
}

impl Resource for Emg {
    fn init() -> Self {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 500_000, Mode::Mode0);
        if spi.is_err() {
            error!("Failed to initialize SPI: {:?}", spi.err());
            panic!("Failed to initialize SPI");
        }

        let emg = Emg {
            spi: spi.unwrap(),
            inner_read_buffer_size: DEFAULT_BUFFER_SIZE,
            outer_read_buffer_size: DEFAULT_BUFFER_SIZE,
            inner_threshold: 0,
            outer_threshold: 0,
        };
        emg
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl ResourceManager for Manager<Emg> {
    run!(Emg);

    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;

        let res: String = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined EMG task type: {:?}", task);
                UNDEFINED_TASK.to_string()
            }
            Task::Idle => {
                match read_adc_channels(&[0, 1], &mut self.metadata.spi) {
                    Ok(value) => {
                        info!("EMG ADC Channel 0 value: {:?}", value);
                        match process_data(value, self.metadata.inner_threshold, self.metadata.outer_threshold) {
                            Ok(grip_state) => {
                                info!("Grip state: {:?}", grip_state);
                                if grip_state == 1 {
                                    info!("Opening hand");
                                    "OPEN HAND".to_string()
                                } else {
                                    info!("Closing hand");
                                    "CLOSE HAND".to_string()
                                }
                            }
                            Err(e) => {
                                error!("Failed to process EMG data: {:?}", e);
                                TASK_FAILURE.to_string()
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from EMG SPI: {:?}", e);
                        format!("Failed to read SPI: {}", e)
                    }
                }
            }
            Task::Calibrate => {
                let thresholds = calibrate_emg(DEFAULT_BUFFER_SIZE, DEFAULT_BUFFER_SIZE, &mut self.metadata.spi);
                self.metadata.inner_threshold = thresholds[0];
                self.metadata.outer_threshold = thresholds[1];
                TASK_SUCCESS.to_string()
            }
            Task::Abort => "Aborting EMG task".to_string(),
        };

        send_channel.send(res).ok();
        Ok(())
    }
}

fn process_data(values: Vec<u16>, inner_threshold: u16, outer_threshold: u16) -> Result<i32, Box<dyn StdError>> {
    if values.len() != 2 {
        return Err("Expected 2 EMG values".into());
    }

    /* 
    if values[0] >= inner_threshold && values[1] <= outer_threshold {
        Ok(1) // Open
    } else {
        Ok(0) // Close
    }
    */

    if values[0] >= inner_threshold{
        Ok(1) // Open
    } else {
        Ok(0) // Close
    }
}

    
fn calibrate_emg(inner_read_buffer_size: usize, outer_read_buffer_size: usize, spi: &mut Spi) -> [u16; 2] {
    let inner_buffer = read_samples(0, inner_read_buffer_size, spi, "inner");

    print!("\nFinished inner sampling. Press ENTER when you're ready to start outer sampling...");
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut String::new());

    let outer_buffer = read_samples(1, outer_read_buffer_size, spi, "outer");

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

fn read_samples(channel: u8, sample_count: usize, spi: &mut Spi, label: &str) -> Vec<u16> {
    let mut buffer = Vec::with_capacity(sample_count);
    println!("Flex {label}");

    while buffer.len() < sample_count {
        match read_adc_channel(channel, spi) {
            Ok(value) => buffer.push(value),
            Err(_) => println!("Error reading SPI on channel {channel} during {label}"),
        }
        thread::sleep(Duration::from_millis(PAUSE_DURATION_MS as u64));
    }

    buffer
}


fn read_adc_channels(channels: &[u8], spi: &mut Spi) -> Result<Vec<u16>, Box<dyn StdError>> {
    channels.iter().map(|&channel| read_adc_channel(channel, spi)).collect()
}

fn read_adc_channel(channel: u8, spi: &mut Spi) -> Result<u16, Box<dyn StdError>> {
    if channel > 7 {
        return Err("Invalid channel. Must be between 0 and 7.".into());
    }

    let start_bit = 0b00000001;
    let config_bits = 0b10000000 | (channel << 4);

    let tx = [start_bit, config_bits, 0x00];
    let mut rx = [0u8; 3];

    spi.transfer(&mut rx, &tx)?;

    let result = ((rx[1] & 0b00000011) as u16) << 8 | (rx[2] as u16);
    Ok(result)
}