mod managers;
mod server;

/// Imports the protobuf generated code to enable de/serialization
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

/// Simple wrapper to allow retrying on failures
#[macro_export]
macro_rules! retry {
    ($f:expr, $count:expr, $interval:expr) => {{
        let mut retries = 0;
        let result = loop {
            let result = $f;
            if result.is_ok() {
                break result;
            } else if retries > $count {
                break result;
            } else {
                retries += 1;
                tokio::time::sleep(std::time::Duration::from_millis($interval)).await;
            }
        };
        result
    }};
    ($f:expr) => {
        retry!($f, 5, 100)
    };
}

/// todo! without panicking
#[macro_export]
macro_rules! todo {
    () => {
        error!("Not yet implemented")
    };
}

/// Certain methods are only run when GPM is in the Raspberry Pi environment (for eg. GPIO access).
/// This macro must be used to log whenever some task is skipped when running GPM outside
/// the Pi.
#[macro_export]
macro_rules! not_on_pi {
    () => {
        warn!("Not running on the Raspberry Pi -- skipping task")
    };
}
