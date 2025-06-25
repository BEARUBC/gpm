#[cfg(feature = "pi")]
mod actual;
#[cfg(not(feature = "pi"))]
mod mock;

#[cfg(feature = "pi")]
pub use actual::run_emg_dispatcher_loop;
#[cfg(not(feature = "pi"))]
pub use mock::run_emg_dispatcher_loop;
