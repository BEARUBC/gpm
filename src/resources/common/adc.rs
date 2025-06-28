// MCP3008 Client
use anyhow::{Context, Error, Result};

use rppal::gpio::OutputPin;
use rppal::spi::Spi;

use rppal::gpio::{Gpio, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

pub struct Adc {
    pub spi: Spi,
    pub cs_pin: OutputPin,
}

impl Adc {
    pub fn init(pin: u8) -> Self {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 500_000, Mode::Mode0)
            .expect("Failed to initialize SPI");

        let mut cs = Gpio::new()
            .expect("Failed to initialize manual CS")
            .get(pin)
            .expect("Failed to get GPIO pin for CS")
            .into_output();

        cs.set_high();

        Adc { spi, cs_pin: cs }
    }

    // Reads the 10-bit ADC value from a given channel (0–7) on the MCP3008 via SPI.
    // MCP3008 messaging protocol: 3 byte message structure
    // doc link: https://www.mathworks.com/help/matlab/supportpkg/analog-input-using-spi.html
    pub fn read_channel(&mut self, channel: u8) -> Result<u16> {
        if channel > 7 {
            return Err(Error::msg(format!(
                "Invalid ADC channel: {}. Must be between 0 and 7.",
                channel
            )));
        }

        // first byte: start sequence, sends 1 as start bit
        let start_bit = 0b00000001;

        // second byte: mode (differential or single read) and channel select
        // sets bit 7 to 1 first, then left-shifts channel number by 4 bits to put it into bits 6,5,4
        // then combines with bitwise OR operator
        // remaining bits are ignored
        let config_bits = 0b10000000 | (channel << 4);

        // third byte: dummy to clock out rest of ADC result
        let tx = [start_bit, config_bits, 0x00];

        // response buffer
        let mut rx = [0u8; 3];

        //activate chip select
        self.cs_pin.set_low();

        // full-duplex SPI transfer: sends tx[], fills rx[]
        self.spi
            .transfer(&mut rx, &tx)
            .context("SPI transfer failed during ADC read")?;

        // deactivate chip select
        self.cs_pin.set_high();

        // adc sends back 3 byte response
        // actual response is 10 bits and spread across rx[1](bits 9-8) and rx[2](bits 7-0)
        // isolate rx[1] result bits, then shift them to 9-8 in a 16 bit number, then add remaining bits by combining them with bitwise OR
        let result = ((rx[1] & 0b00000011) as u16) << 8 | (rx[2] as u16);

        Ok(result)
    }

    pub fn read_channels(&mut self, channels: &[u8]) -> Result<Vec<u16>> {
        channels
            .iter()
            .map(|&channel| {
                self.read_channel(channel)
                    .with_context(|| format!("Failed to read from ADC channel {}", channel))
            })
            .collect()
    }

    // Averages ADC readings
    pub fn average_values(list: &Vec<u16>) -> Result<u16> {
        if list.is_empty() {
            return Err(Error::msg("Cannot calculate average of an empty list"));
        } else {
            let sum: u32 = list.iter().map(|&x| x as u32).sum();
            Ok((sum / list.len() as u32) as u16)
        }
    }
}
