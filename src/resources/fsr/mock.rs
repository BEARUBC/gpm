// All tasks operating on the EMG system live in this file
use crate::resources::Resource;
use crate::sgcp;

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
