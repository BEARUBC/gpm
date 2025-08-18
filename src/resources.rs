pub mod bms;
pub mod common;
pub mod emg;
pub mod fsr;
pub mod maestro;

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
