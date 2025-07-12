use crate::resources::Resource;
use crate::sgcp;

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