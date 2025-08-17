// All tasks operating on the EMG system live in this file
use gpm::sgcp;

use crate::resources::Resource;

// TODO: Implement mock Fsr

pub struct Fsr;

impl Resource for Fsr {
    fn init() -> Self {
        Fsr {}
    }

    fn name() -> String {
        sgcp::Resource::Fsr.as_str_name().to_string()
    }
}
