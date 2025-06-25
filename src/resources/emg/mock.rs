// All tasks operating on the EMG system live in this file
use crate::config::Config;
use crate::managers::Manager;
use crate::managers::ManagerChannelData;
use crate::managers::ResourceManager;
use crate::managers::macros::parse_channel_data;
use crate::request::TaskData::EmgData;
use crate::resources::Resource;
use crate::sgcp;
use crate::sgcp::emg::*;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use log::*;

// TODO: Implement mock Emg

pub struct Emg;

impl Resource for Emg {
    fn init() -> Self {
        Emg {}
    }

    fn name() -> String {
        sgcp::Resource::Emg.as_str_name().to_string()
    }
}
