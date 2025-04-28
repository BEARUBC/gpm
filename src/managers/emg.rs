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
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};
use std::char::DecodeUtf16;
use std::io;
use mcp3008::{Mcp3008, Mcp3008Error};
use cyclic_list::List;
use std::iter::FromIterator;

/// Represents an EMG resource
pub struct calibrationVisualizer{

}

impl calibrationVisualizer{
    fn init() -> (){

    }
}
fn average(list: Vec<u16>)-> u16{
    if list.is_empty() {
        return 0;
    }
    let sum: u16 = list.iter().map(|&x| x as u16).sum();
    sum as u16 / list.len() as u16
}

pub struct Emg {
    adc: Option<Mcp3008>,
    inner_read_buffer_size: usize,
    outer_read_buffer_size: usize,
    sleep_between_reads_in_seconds: f32,
    use_mock_adc: bool,
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
            inner_read_buffer_size: 2000,
            outer_read_buffer_size: 2000,
            sleep_between_reads_in_seconds: 0.1,
            use_mock_adc: false,
            inner_threshold: 0,
            outer_threshold: 0,
        };
        
        let thresholds = calibrate_emg(emg.inner_read_buffer_size, emg.outer_read_buffer_size);
        emg.inner_threshold = thresholds[0];
        emg.outer_threshold = thresholds[1];
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
                warn!("Encountered an undefined EMG task type");
                UNDEFINED_TASK.to_string()
            }

            Task::Idle => {
                match read_adc_channels([0, 1]) { // self is not an emg object it's the manager
                    Ok(value) => {
                        info!("EMG ADC Channel 0 value: {}", value);
                        
                        match process_data(value){
                
                        }
                        // if process data, 
                        TASK_SUCCESS.to_string()
                    }
                    Err(e) => {
                        error!("Failed to read from EMG ADC: {:?}", e);
                        format!("Failed to read ADC: {}", e)
                    }
                }
            }
            Task::Calibrate => {
                let thresholds = emg.calibrate_emg(2000, 2000);
                emg.inner_threshold = thresholds[0];
                emg.outer_threshold = thresholds[1];
            }
            Task::Abort => {
                ABORT.to_string()
            }
        };

        send_channel.send(res).ok();
        Ok(())
    }
}

fn process_data(values: Vec<u16>, inner_threshold: u16, outer_threshold:u16) -> Result<GripState, Box<dyn std::error::Error>> {
    if values.len() != 2 {
        return Err("Expected 2 EMG values".into());
    }

    if values[0] >= inner_threshold && values[1] <= outer_threshold {
        Ok(GripState::Open)
    } else {
        Ok(GripState::Closed)
    }
}
    
fn calibrate_emg(inner_read_buffer_size: usize, outer_read_buffer_size:usize, &mut adcOption: Option<Mcp3008>) -> [u16; 2] {
    // read and populate the buffer
    let mut inner_buffer: Vec<u16> = vec![0; inner_read_buffer_size];
    let mut outer_buffer: Vec<u16> = vec![0; outer_read_buffer_size];
    let mut output: [u16; 2] = [0; 2];
    let mut i = 0;
    let mut j = 0;
    println!("Flex inner");
    while inner_buffer[inner_read_buffer_size] == 0 {
        let adc_cal = read_adc_channel(0, &mut adcOption);
        match adc_cal{
            Ok(v) => inner_buffer[i] = v,
            Err(e) => println!("Error reading adc inner"),
        } 
        i += 1;
        // add delay if needed
    }
    output[0] = average(inner_buffer);
    println!("Flex outer");
    while outer_buffer[outer_read_buffer_size] == 0 {
        let adc_cal = read_adc_channel(1);
        match adc_cal{
            Ok(v) => outer_buffer[j] = v,
            Err(e) => println!("Error reading adc outer"),
        }
        j += 1;   
    }
    output[1] = average(outer_buffer);
    return output;
}
fn read_adc_channels(channels: &[u8], &mut adcOption: Option<Mcp3008>) -> Result<Vec<u16>, Mcp3008Error> {
    let adc = adcOption.as_mut().unwrap();
    let mut output = Vec::with_capacity(channels.len());
    for &channel_num in channels {
        let value = adc.read_adc(channel_num)?;
        output.push(value);
    }
    Ok(output)
}
fn read_adc_channel(channel: u8, &mut adcOption: Option<Mcp3008>) -> Result<u16, Mcp3008Error> {
    let adc = adcOption.as_mut().unwrap();
    let output = adc.read_adc(channel);
    return output;
}

fn start_reading_adc() -> Result<Mcp3008, Mcp3008Error> {
    let path = "/dev/spidev0.0";
    Mcp3008::new(path)
}