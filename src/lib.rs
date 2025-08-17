mod macros;

pub mod sgcp {
    include!(concat!(env!("OUT_DIR"), "/sgcp.rs"));
    pub mod bms {
        include!(concat!(env!("OUT_DIR"), "/sgcp.bms.rs"));
    }
    pub mod emg {
        include!(concat!(env!("OUT_DIR"), "/sgcp.emg.rs"));
    }
    pub mod maestro {
        include!(concat!(env!("OUT_DIR"), "/sgcp.maestro.rs"));
    }
    pub mod fsr {
        include!(concat!(env!("OUT_DIR"), "/sgcp.fsr.rs"));
    }
}

use sgcp::Resource;

/// Given a sgcp Resource enum variant, returns a vector containing string-names of
/// the tasks available on that resource
pub fn get_tasks_for_resource(resource: &sgcp::Resource) -> Vec<String> {
    match resource {
        Resource::UndefinedComponent => {
            panic!("There are not tasks associated to the undefined component")
        },
        Resource::Bms => get_task_names!(sgcp::bms::Task),
        Resource::Emg => get_task_names!(sgcp::emg::Task),
        Resource::Maestro => get_task_names!(sgcp::maestro::Task),
        Resource::Fsr => get_task_names!(sgcp::fsr::Task),
    }
}
