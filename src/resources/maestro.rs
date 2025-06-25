#[cfg(feature = "pi")]
mod actual;
#[cfg(not(feature = "pi"))]
mod mock;

#[cfg(feature = "pi")]
pub use actual::Maestro;
#[cfg(not(feature = "pi"))]
pub use mock::Maestro;
