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
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use std::char::DecodeUtf16;
use std::io;
use std::io::Write;
use mcp3008::{Mcp3008, Mcp3008Error};
use cyclic_list::List;
use std::iter::FromIterator;
use anyhow::anyhow;

const DEFAULT_BUFFER_SIZE: usize = 100;
const SPI_DEVICE_PATH: &str = "/dev/spidev0.0";


/// Represents an EMG resource
pub struct calibrationVisualizer{

}

impl calibrationVisualizer{
    fn init() -> (){

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


pub struct Emg {
    adc: Option<Mcp3008>,
    inner_read_buffer_size: usize,
    outer_read_buffer_size: usize,
    inner_threshold: u16,
    outer_threshold: u16,
}


pub struct fakeEMG{
    // idk read some data from a csv
}

impl Resource for Emg {
    fn init() -> Self {
        let adc = start_reading_adc().ok();

        let mut emg = Emg {
            adc,
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
    run!(Emg); // note to self, this in an impl block, this doesnt run before handle_task
    
    /// Handles all EMG-related tasks // is this meant to be for the 
    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;

        // clear the buffer
        let res: String = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined EMG task type: {:?}", task);
                UNDEFINED_TASK.to_string()
            }

            Task::Idle => {
                match read_adc_channels(&[0, 1], &mut self.metadata.adc) { // self is not an emg object it's the manager
                    Ok(value) => {
                        info!("EMG ADC Channel 0 value: {:?}", value);
                        match process_data(value, self.metadata.inner_threshold, self.metadata.outer_threshold) {
                            Ok(grip_state) => {
                                info!("Grip state: {:?}", grip_state); 
                                if grip_state == 1 {
                                    // open hand
                                    info!("Opening hand");
                                    "OPEN HAND".to_string();
                                } else {
                                    // close hand
                                    info!("Closing hand");
                                    "CLOSE HAND".to_string();
                                }
                            }
                            Err(e) => {
                                error!("Failed to process EMG data: {:?}", e);
                            }
                
                        }
                        // if process data, 
                        TASK_SUCCESS.to_string()
                    }
                    Err(e) => {
                        error!("Failed to read from EMG ADC: {:?}", e);
                        format!("Failed to read ADC: {}", e);
                        TASK_FAILURE.to_string()
                    }
                }
            }
            Task::Calibrate => {
                let thresholds = calibrate_emg(DEFAULT_BUFFER_SIZE, DEFAULT_BUFFER_SIZE, &mut self.metadata.adc);
                // add error handling for thresholds
                self.metadata.inner_threshold = thresholds[0];
                self.metadata.outer_threshold = thresholds[1];
                TASK_SUCCESS.to_string()
            }
            Task::Abort => {
                "Aborting EMG task".to_string()
            }
        };

        send_channel.send(res).ok();
        Ok(())
    }
}

// replace gripstate with integer or something
fn process_data(values: Vec<u16>, inner_threshold: u16, outer_threshold:u16) -> Result<i32, Box<dyn std::error::Error>> {
    if values.len() != 2 {
        return Err("Expected 2 EMG values".into());
    }

    if values[0] >= inner_threshold && values[1] <= outer_threshold {
        Ok(1) // Open
    } else {
        Ok(0) // Close
    }
}
    
fn calibrate_emg(inner_read_buffer_size: usize, outer_read_buffer_size: usize, adc_option: &mut Option<Mcp3008>) -> [u16; 2] {
    let inner_buffer = read_samples(0, inner_read_buffer_size, adc_option, "inner");
    
    // Prompt the user to continue
    print!("\nFinished inner sampling. Press ENTER when you're ready to start outer sampling...");
    io::stdout().flush().unwrap(); // Ensure prompt is printed
    let _ = io::stdin().read_line(&mut String::new());

    let outer_buffer = read_samples(1, outer_read_buffer_size, adc_option, "outer");

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

fn read_samples(
    channel: u8,
    sample_count: usize,
    adc_option: &mut Option<Mcp3008>,
    label: &str,
) -> Vec<u16> {
    let mut buffer = Vec::with_capacity(sample_count);
    println!("Flex {label}");

    while buffer.len() < sample_count {
        match read_adc_channel(channel, adc_option) {
            Ok(value) => buffer.push(value),
            Err(_) => println!("Error reading ADC on channel {channel} during {label}"),
        }
    }

    buffer
}


fn read_adc_channels(channels: &[u8], adc_option: &mut Option<Mcp3008>) -> Result<Vec<u16>, Mcp3008Error> {
    channels
        .iter()
        .map(|&channel| read_adc_channel(channel, adc_option))
        .collect()
}

fn read_adc_channel(channel: u8, adc_option: &mut Option<Mcp3008>) -> Result<u16, Mcp3008Error> {
    let adc = adc_option.as_mut().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "ADC not initialized"))?;
    adc.read_adc(channel)
}

fn start_reading_adc() -> Result<Mcp3008, Mcp3008Error> {
    let path = SPI_DEVICE_PATH;
    Mcp3008::new(path)
}