use crate::resources::Resource;
use crate::sgcp;

pub struct Maestro;

impl Resource for Maestro {
    fn init() -> Self {
        Maestro {}
    }

    fn name() -> String {
        sgcp::Resource::Maestro.as_str_name().to_string()
    }
}
