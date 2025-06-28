use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::resources::emg::Emg;
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

                let grip_state = self.resource.process_data(adc_values)?;
                info!("Grip state: {:?}", grip_state);

                if grip_state == 1 {
                    info!("Opening hand");
                    Ok("OPEN HAND".to_string())
                } else {
                    // TODO: handle the case where grip_state is -1
                    info!("Closing hand");
                    Ok("CLOSE HAND".to_string())
                }
            },
            Task::Calibrate => match self.resource.calibrate_emg() {
                Ok(_) => Ok(TASK_SUCCESS.to_string()),
                Err(e) => {
                    error!("Calibration failed: {:?}", e);
                    Err(Error::msg(format!("Calibration failed: {}", e)))
                },
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
