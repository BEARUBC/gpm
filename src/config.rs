use std::env::set_var;

pub const MAX_TCP_CONNECTIONS: usize = 1;
pub const GPM_TCP_ADDR: &str = "127.0.0.1:4760";
pub const TELEMETRY_TCP_ADDR: &str = "127.0.0.1:9999";

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
