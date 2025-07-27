#[cfg(feature = "pi")]
#[path = "emg/actual.rs"]
mod emg_impl;

#[cfg(not(feature = "pi"))]
#[path = "emg/mock.rs"]
mod emg_impl;

pub use emg_impl::Emg;
