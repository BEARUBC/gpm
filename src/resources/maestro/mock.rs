// All tasks operating on the Maestro servo controller live in this file

use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::macros::parse_channel_data;
use crate::not_on_pi;
use crate::request::TaskData::MaestroData;
use crate::resources::Resource;
use crate::sgcp;
use crate::sgcp::maestro::Task as MaestroTask;
use crate::sgcp::maestro::*;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;
use std::time::Duration;

/// Represents a Maestro resource
pub struct Maestro;

impl Resource for Maestro {
    fn init() -> Self {
        Maestro {}
    }

    fn name() -> String {
        sgcp::Resource::Maestro.as_str_name().to_string()
    }
}
