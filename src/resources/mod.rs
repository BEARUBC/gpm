pub mod bms;
pub mod emg;
pub mod maestro;

pub trait Resource {
    fn init() -> Self;
    fn name() -> String;
}
