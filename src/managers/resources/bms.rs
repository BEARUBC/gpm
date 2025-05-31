// All tasks operating on the BMS (Battery Management System) live in this file
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::Resource;
use crate::managers::ResourceManager;
use crate::managers::TASK_SUCCESS;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::BmsData;
use crate::sgcp;
use crate::sgcp::bms::*;
use crate::todo;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;

/// Represents a BMS resource
pub struct Bms {
    // TODO: @krarpit Implement BMS interface
    // BMS health report by Elin
    pub voltage: f32,
    pub temperature: f32,
    pub charge_percent: u8,
    pub status: String,
}

impl Resource for Bms {
    fn init() -> Self {
        Bms {} // stub
    }

    fn name() -> String {
        sgcp::Resource::Bms.as_str_name().to_string()
    }
}

impl ResourceManager for Manager<Bms> {
    run!(Bms);

    /// Handles all BMS-related tasks
    async fn handle_task(&mut self, channel_data: ManagerChannelData) -> Result<()> {
        let (task, task_data, send_channel) =
            parse_channel_data!(channel_data, Task, BmsData).map_err(|e: Error| e)?;
        
        //these are sample data!
        let health_report = BmsHealthReport {
            voltage: 12.6,
            temperature: 29.4,
            charge_percent: 82,
            status: "nominal".to_string(),
        };
        
        // To do: Add ""::Responder::respond()"" 
        // or convert the code below to "send_channel.send(...)" 
        // -> refer to Krisha's code below
        Responder::respond(health_report, send_channel)?;

        match task {
            Task::UndefinedTask => todo!(),
            Task::GetHealthMetrics => todo!(),
            Task::GetChargeStatus => todo!(),
            Task::ShutDownBattery => todo!(),
        }
        send_channel.send(TASK_SUCCESS.to_string());
        Ok(())
    }
}

