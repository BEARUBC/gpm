use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use gpm::sgcp::emg::*;
use gpm::sgcp::request::TaskData::EmgData;
use log::*;
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
use rppal::spi::Bus;
use rppal::spi::Mode;
use rppal::spi::SlaveSelect;
use rppal::spi::Spi;

use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::resources::emg::Emg;

impl ResourceManager for Manager<Emg> {
    type ResourceType = Emg;

    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, _task_data, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;

        let res = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            },
            Task::Idle => {
                let adc_values = self.resource.read_adc_channels(&[0, 1])?;
                info!("EMG ADC Channel 0,1 value: {:?}", adc_values);

                loop { // loops to fill EMG buffer for processing
                    if !self.resource.emg_processor_inner.window_filled || !self.resource.emg_processor_outer.window_filled {
                        let adc_values = self.resource.read_adc_channels(&[0, 1])?;
                        if let Some(_processed_inner) = self.resource.emg_processor_inner.process_sample(adc_values[0]) {
                            info!("Inner EMG window filled");
                        }
                        if let Some(_processed_outer) = self.resource.emg_processor_outer.process_sample(adc_values[1]) {
                            info!("Outer EMG window filled");
                        }
                    } else {
                        info!("EMG ADC Channel 0,1 value: {:?}", adc_values);
                        break;
                    }
                }
                
                let grip_state = self.resource.process_data(adc_values)?;
                info!("Grip state: {:?}", grip_state);

                if grip_state == 1 {
                    info!("Opening hand");
                    Ok("OPEN HAND".to_string())
                } else {
                    info!("Closing hand");
                    Ok("CLOSE HAND".to_string())
                }
            },
            Task::Calibrate => {
                Emg::calibrate_emg(&mut self.resource)?;
                info!("EMG Calibrated. Inner Threshold: {}, Outer Threshold: {}", self.resource.inner_threshold, self.resource.outer_threshold);
                Ok(TASK_SUCCESS.to_string())
            },
            Task::Abort => {
                info!("Aborting EMG task");
                Ok(TASK_SUCCESS.to_string())
            },
        };

        let response = match res {
            Ok(message) => message,
            Err(e) => format!("Error: {e}"),
        };

        Ok(send_channel
            .send(response)
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}
