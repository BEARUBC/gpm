#[cfg(feature = "pi")]
#[path = "emg/actual.rs"]
mod emg_impl;

#[cfg(not(feature = "pi"))]
#[path = "emg/mock.rs"]
mod emg_impl;

#[derive(serde::Serialize)]
pub struct EmgData {
    pub channel_0: f64,
    pub channel_1: f64,
    pub timestamp: u64,
}

pub use emg_impl::Emg;
