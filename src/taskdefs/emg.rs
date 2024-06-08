// All tasks operating on the EMG system will live in this file.
// We are using the LTC1865 ADC
use anyhow::Result;

// Stub
pub async fn read_edc() -> Result<()> {
	println!("emg::read_edc called");
	Ok(())
}