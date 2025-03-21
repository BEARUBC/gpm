// All tasks operating on the EMG system live in this file
// NOTE: All EMG proccessing will now be handled on the Jetson Nano instead of GPM. Leaving
//       this file here in case we decide to implement an EMG interface for GPM anyway.

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
use crate::managers::resources::adc;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;


use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;



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
    type ResourceType = Emg;

    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;

        let res = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            }
            Task::Idle => {
                match adc::read_adc_channels(&[0, 1], &mut self.resource.spi) {
                    Ok(value) => {
                        info!("EMG ADC Channel 0 value: {:?}", value);
                        match adc::process_data(value, self.resource.inner_threshold, self.resource.outer_threshold) {
                            Ok(grip_state) => {
                                info!("Grip state: {:?}", grip_state);
                                if grip_state == 1 {
                                    info!("Opening hand");
                                    Ok("OPEN HAND".to_string())
                                } else {
                                    info!("Closing hand");
                                    Ok("CLOSE HAND".to_string())
                                }
                            }
                            Err(e) => {
                                error!("Failed to process EMG data: {:?}", e);
                                Err(Error::msg("Failed to process EMG data: {}"))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read from EMG SPI: {:?}", e);
                        Err(Error::msg("Failed to read SPI"))
                    }
                }
            }
            Task::Calibrate => {
                let thresholds = adc::calibrate_emg(DEFAULT_BUFFER_SIZE, DEFAULT_BUFFER_SIZE, &mut self.resource.spi);
                self.resource.inner_threshold = thresholds[0];
                self.resource.outer_threshold = thresholds[1];
                Ok(TASK_SUCCESS.to_string())
            }
            Task::Abort => {
                info!("Aborting EMG task");
                Ok(TASK_SUCCESS.to_string())
            }
        };

        let response = match res {
            Ok(message) => {
                if message == "OPEN HAND" || message == "CLOSE HAND" {
                    message
                } else {
                    TASK_SUCCESS.to_string()
                }
            }
            Err(e) => format!("Error: {e}"),
        };

        Ok(send_channel
            .send(response)
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}


