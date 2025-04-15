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
    adc: Option<Mcp3008>,
    inner_read_buffer_size: usize,
    outer_read_buffer_size: usize,
    sleep_between_reads_in_seconds: f32,
    use_mock_adc: bool,
    upper_threshold: f32,
    lower_threshold: f32
}


pub struct fakeEMG{
    // idk read some data from a csv
}

impl Resource for Emg {
    fn init() -> Self {
        let adc = Self::start_reading_adc().ok();

        let emg = Emg {
            adc,
            inner_read_buffer_size: 2000,
            outer_read_buffer_size: 2000,
            sleep_between_reads_in_seconds: 0.1,
            use_mock_adc: false,
            upper_threshold: None,
            lower_threshold: None
        };

        emg.calibrate_emg();
        emg
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}

impl ResourceManager for Manager<Emg> {
    run!(Emg);

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

            Task::ProcessDataTask => {
                match self.resource.read_adc(0) {
                    Ok(value) => {
                        info!("EMG ADC Channel 0 value: {}", value);
                        // TODO: Use value to interpret grip intent and trigger downstream tasks
                        self.process_data(); 
                        // if process data, 
                        TASK_SUCCESS.to_string()
                    }
                    Err(e) => {
                        error!("Failed to read from EMG ADC: {:?}", e);
                        format!("Failed to read ADC: {}", e)
                    }
                }
            }
        };

        send_channel.send(res).ok();
        Ok(())
    }
}

impl Emg{

    fn process_data(&self) -> Result<()>{
        // take in buffer values, if average is greater than calibration
        Ok(())
    }
    
    fn calibrate_emg(&self) -> [i32; 2] {
        // read and populate the buffer
        let inner_buffer = u16[self.inner_read_buffer_size];
        let outer_buffer = u16[self.inner_read_buffer_size];
        let i = 0;
        let j = 0;
        println!("Flex inner");
        while inner_buffer[self.inner_read_buffer_size] == None {
            let adc_cal = self.read_adc(0);
            match adc_cal{
                Ok(v) => inner_buffer[i],
                Err(e) => println!("Error reading adc inner"),
            }
            i += 1;
        }
        // take average of this   
        println!("Flex outer");
        while outer_buffer[self.outer_read_buffer_size] == None {
            let adc_cal = self.read_adc(1);
            match adc_cal{
                Ok(v) => outer_buffer[j],
                Err(e) => println!("Error reading adc outer"),
            }
            j += 1;   
        }
        // take average of this
    }

    fn read_adc(&self, channel: u8) -> Result<u16, Mcp3008Error> {
        self.adc
            .as_ref()
            .ok_or_else(|| Mcp3008Error::Spi(std::io::Error::new(std::io::ErrorKind::Other, "ADC not initialized")))?
            .read_channel(channel)
    }

    fn start_reading_adc() -> Result<Mcp3008<Spidev>, Box<dyn std::error::Error>> {
        let mut spi = Spidev::open("/dev/spidev0.0")?;
        let options = spidev::SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(1_000_000)
            .mode(spidev::SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;
        Ok(Mcp3008::new(spi))
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
