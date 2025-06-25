// All tasks operating on the EMG system live in this file
use crate::config::Config;
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::resources::Resource;
use crate::resources::emg::Emg;
use crate::sgcp;
use crate::sgcp::emg::*;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;

use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

impl ResourceManager for Manager<Emg> {
    type ResourceType = Emg;

    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(channel_data, Task, EmgData).map_err(|e: Error| e)?;

        let res = match task {
            Task::UndefinedTask => {
                warn!("Encountered an undefined task type");
                Err(Error::msg("Encountered an undefined task type"))
            },
            Task::Idle => {
                let adc_values = Emg::read_adc_channels(
                    &[0, 1],
                    &mut self.resource.cs_pin,
                    &mut self.resource.spi,
                )?;
                info!("EMG ADC Channel 0,1 value: {:?}", adc_values);

                let grip_state = Emg::process_data(
                    adc_values,
                    self.resource.inner_threshold,
                    self.resource.outer_threshold,
                )?;
                info!("Grip state: {:?}", grip_state);

                if grip_state == 1 {
                    info!("Opening hand");
                    Ok("OPEN HAND".to_string())
                } else {
                    // todo handle the case where grip_state is -1
                    info!("Closing hand");
                    Ok("CLOSE HAND".to_string())
                }
            },
            Task::Calibrate => {
                match adc::calibrate_emg(
                    self.resource.buffer_size,
                    &mut self.resource.spi,
                    &mut self.resource.cs_pin,
                    self.resource.inter_channel_sample_duration,
                ) {
                    Ok(thresholds) => {
                        self.resource.inner_threshold = thresholds[0];
                        self.resource.outer_threshold = thresholds[1];
                        Ok(TASK_SUCCESS.to_string())
                    },
                    Err(e) => {
                        error!("Calibration failed: {:?}", e);
                        Err(Error::msg(format!("Calibration failed: {}", e)))
                    },
                }
            },
            Task::Abort => {
                info!("Aborting EMG task");
                Ok(TASK_SUCCESS.to_string())
            },
        };

        let response = match res {
            Ok(message) => {
                if message == "OPEN HAND" || message == "CLOSE HAND" {
                    message
                } else {
                    TASK_SUCCESS.to_string()
                }
            },
            Err(e) => format!("Error: {e}"),
        };

        Ok(send_channel
            .send(response)
            .map_err(|e| anyhow!("Send Failed: {e}"))?)
    }
}
