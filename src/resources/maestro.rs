#[cfg(feature = "pi")]
#[path = "maestro/actual.rs"]
mod maestro_impl;

#[cfg(not(feature = "pi"))]
#[path = "maestro/mock.rs"]
mod maestro_impl;

pub use maestro_impl::Maestro;
