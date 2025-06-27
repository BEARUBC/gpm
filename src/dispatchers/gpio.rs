#[cfg(feature = "pi")]
mod actual;
#[cfg(not(feature = "pi"))]
mod mock;

pub struct GpioDispatcher;
