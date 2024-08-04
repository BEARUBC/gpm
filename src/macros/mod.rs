mod managers;
mod server;

#[macro_export]
macro_rules! import_sgcp {
    () => {
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
        }
        use sgcp::*;
    };
}

#[macro_export]
macro_rules! todo {
    () => {
        error!("Not yet implemented")
    };
}

#[macro_export]
macro_rules! not_on_pi {
    () => {
        warn!("Not running on the Raspberry Pi, assume success")
    }
}
