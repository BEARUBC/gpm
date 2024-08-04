use std::env::set_var;

pub const MAX_CONCURRENT_CONNECTIONS: usize = 1;
pub const GPM_TCP_ADDR: &str = "127.0.0.1:4760";
pub const TELEMETRY_TCP_ADDR: &str = "127.0.0.1:9999";
pub const READ_BUFFER_CAPACITY: usize = 1024;
pub const FRAME_PREFIX_LENGTH: usize = 8;

const RUST_LOG_KEY: &str = "RUST_LOG";

fn print_ascii() {
    println!(
        r"
   ______                    
  / ____/________ __________ 
 / / __/ ___/ __ `/ ___/ __ \
/ /_/ / /  / /_/ (__  ) /_/ /
\____/_/   \__,_/____/ .___/ 
                    /_/      "
    );
    println!("Version 0.0.1");
    println!("Developed at UBC Bionics");
}

pub fn init() {
    set_var(RUST_LOG_KEY, "trace");
    env_logger::init();
    print_ascii();
}
