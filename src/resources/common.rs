#[cfg(feature = "pi")]
pub mod adc;

#[cfg(feature = "pi")]
pub use adc::Adc;
