#[cfg(feature = "pi")]
#[path = "fsr/actual.rs"]
mod fsr_impl;

#[cfg(not(feature = "pi"))]
#[path = "fsr/mock.rs"]
mod fsr_impl;

pub use fsr_impl::Fsr;
