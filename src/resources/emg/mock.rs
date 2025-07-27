// All tasks operating on the EMG system live in this file
use crate::resources::Resource;
use gpm::sgcp;

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
