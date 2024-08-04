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
    };
}
