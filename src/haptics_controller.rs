// use std::error::Error;

#[cfg(feature = "pi")]
use rppal::spi::Bus;
#[cfg(feature = "pi")]
use rppal::spi::Mode;
#[cfg(feature = "pi")]
use rppal::spi::SlaveSelect;
#[cfg(feature = "pi")]
use rppal::spi::Spi;

pub fn start() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "pi")]
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_350_000, Mode::Mode0)?;

    let tx_buf: [u8; 3] = [
        0b00000001, // Start bit
        0b10000000, // ask adc to read from ch0 (for now)
        0b00000000, // junk
    ];

    let mut rx_buf = [0u8; 3];

    // asserting/deasserting CS is handle by transfer() automatically
    let tsize = spi.transfer(&mut rx_buf, &tx_buf)?;
    print!("Size of transfer: {tsize}\n");

    // adc result is a 10-bit unsigned int
    // last 2 bits of rx_buf[1] has to be the high bits of the 10-bit result
    // rx_buf[2] is the low bits of the 10-bit adc result
    let result: u16 = ((rx_buf[1] as u16 & 0x3 as u16) << 8) | rx_buf[2] as u16;
    print!("ADC result is: {result}\n");

    return Ok(());
}
