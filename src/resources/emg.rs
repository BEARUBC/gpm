#[cfg(feature = "pi")]
#[path = "emg/actual.rs"]
mod emg_impl;

#[cfg(not(feature = "pi"))]
#[path = "emg/mock.rs"]
mod emg_impl;

trait AdcReader {
    fn read_adc(&self, channel: u8, label: &str) -> Vec<u16>;
}

pub use emg_impl::Emg;
