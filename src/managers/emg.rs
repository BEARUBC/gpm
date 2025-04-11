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
use mcp3008::Mcp3008;
use embedded_hal::spi::FullDuplex;
use embedded_hal::digital::v2::OutputPin;
use linux_embedded_hal::{Spidev, Pin, SpidevOptions};
use std::error::Error;


/// Represents an EMG resource
pub struct Emg {
    inner_read_buffer_size: u32,
    outer_read_buffer_size: u32,
    mock_reader_state_buffer_size: u32, 
    sleep_between_reads_in_seconds: f16, 
    use_mock_adc: bool,
    adc: Mcp3008
    // circular buffer
    // chan0,1

}

pub struct fakeEMG{
    // idk read some data from a csv
}

impl Resource for Emg {
    fn init() -> Self {   
        Emg {
            inner_read_buffer_size: 2000, // hardcode for now // chekc if we actually need this
            outer_read_buffer_size: 2000,
            mock_reader_state_buffer_size: 100, 
            sleep_between_reads_in_seconds: 0.1, 
            use_mock_adc: false,
            adc: None
        };
        Emg::adc = Emg::start_reading_adc();
        Emg::calibrate_EMG(&self)
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl ResourceManager for Manager<Emg> {
    run!(Emg);

    /// Handles all EMG-related tasks // is this meant to be for the 
    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let channel0_value = Emg::read_adc(&self, 0);
        match channel0_value {
            // Ok() => // send this to maestro manager
            // err() => send a 0 or null to maestro manager
        }
        


        let (task, task_data, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;
        match task {
            Task::UndefinedTask => todo!(),
            Task::ProcessDataTask => todo!()
        }
        send_channel.send(TASK_SUCCESS.to_string());
        Ok(())
    }
}

impl Emg{

    fn process_data(&self) -> Result<()>{
        Ok(())
    }
    
    fn calibrate_EMG(&self) -> [i32; 2]{
        let min_value = 0;
        let max_value = 1023;
        [min_value, max_value]
    }

    fn read_adc(&self, channel: u16) -> Result<u16, Mcp3008Error> {
        // // example read 
        let adc_value = self.adc.read_channel(channel)?;
        println!("ADC value on channel {}: {}", channel, adc_value); // adc_value is u16 
        // while true, append to circular buffer
    } // need result enum to emit grip state

    fn start_reading_adc() -> Mcp3008{
        // create spi bus // move this into handle task
        let mut spi = Spidev::open("/dev/spidev0.0")?;

        let options = SpidevOptions::new() // tweak these
            .bits_per_word(8)
            .max_speed_hz(1_000_000) // 1 MHz
            .mode(MODE)
            .build();
        spi.configure(&options)?;

        // Create MCP3008 instance
        let mut adc = Mcp3008::new(spi);
    }

    fn plot_data(&self) -> Result<()>{
        Ok(())
    }
}

pub struct calibrationVisualizer{

}

impl calibrationVisualizer{
    fn init() -> (){

    }
}
