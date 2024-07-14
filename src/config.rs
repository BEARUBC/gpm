use std::env::set_var;

const RUST_LOG_KEY: &str = "RUST_LOG";

fn print_ascii() {
    println!(r"
   ______                    
  / ____/________ __________ 
 / / __/ ___/ __ `/ ___/ __ \
/ /_/ / /  / /_/ (__  ) /_/ /
\____/_/   \__,_/____/ .___/ 
                    /_/      ");
    println!("Version 0.0.1");
    println!("Developed at UBC Bionics");
}

pub fn init() {
    set_var(RUST_LOG_KEY, "trace");
    pretty_env_logger::init();
    print_ascii();
}
