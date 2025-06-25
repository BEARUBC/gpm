// All tasks operating on the Maestro servo controller live in this file

#![allow(unused_imports)] // Silence warnings because of cfg-gated code

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
#[cfg(feature = "pi")]
use raestro::maestro::{
    builder::Builder,
    constants::{Baudrate, Channel, MAX_QTR_PWM, MIN_QTR_PWM},
};
use std::time::Duration;

/// Represents a Maestro resource
pub struct Maestro {
    #[cfg(feature = "pi")]
    controller: raestro::maestro::Maestro,
}

impl Resource for Maestro {
    fn init() -> Self {
        #[cfg(feature = "pi")]
        {
            let mut controller: raestro::maestro::Maestro = Builder::default()
                .baudrate(Baudrate::Baudrate11520)
                .block_duration(Duration::from_millis(100))
                .try_into()
                .expect("Could not initialize Raestro");
            Maestro { controller }
        }
        #[cfg(not(feature = "pi"))]
        Maestro {}
    }

    fn name() -> String {
        sgcp::Resource::Maestro.as_str_name().to_string()
    }
}
