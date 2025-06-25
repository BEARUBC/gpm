// All tasks operating on the BMS (Battery Management System) live in this file
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::BmsData;
use crate::resources::Resource;
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
}

impl Resource for Bms {
    fn init() -> Self {
        Bms {} // stub
    }

    fn name() -> String {
        sgcp::Resource::Bms.as_str_name().to_string()
    }
}
