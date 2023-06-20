// spi_25aa1024.rs - Transfers data to a Microchip 25AA1024 serial EEPROM using SPI.

use std::error::Error;

use mcp3208::{Channel, Mcp3208};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

/// Number of bits to be sent/received within a single transaction
const FRAME_BIT_COUNT: u8 = 32;

/// number of start bits (always 1)
const START_BIT_COUNT: u8 = 1;

/// index of first (and only) start bit (always the MSB bit)
const START_BIT_INDEX: u8 = FRAME_BIT_COUNT - START_BIT_COUNT; // 31

/// number of bits to select the mode (single or differential)
const MODE_BIT_COUNT: u8 = 1;

/// index of the first (and only) mode selection bit
const MODE_BIT_INDEX: u8 = START_BIT_INDEX - MODE_BIT_COUNT; // 30

/// number of bits required to encode the selected channel
const CHANNEL_BIT_COUNT: u8 = 3;

/// index of the first bit of the channel selection field
const CHANNEL_BITS_INDEX: u8 = MODE_BIT_INDEX - CHANNEL_BIT_COUNT; // 27

pub fn read_adc() -> Result<(), Box<dyn Error>> {
    // outputs the raw adc values of all channels
    /*if let Ok(mut mcp3208) = Mcp3208::new("/dev/spidev0.0") {
        Channel::VALUES.iter().for_each(|&channel| {
            println!(
                "channel #{}: {}",
                channel as u8,
                mcp3208.read_adc_single(channel).unwrap()
            );
        });
    } else {
        println!("Could not do stuff");
    }*/

    fn create_write_buffer(channel: u8) -> [u8; 4] {
        // pattern:
        //   smcccw0r_rrrrrrrr_rrrxxxxx_xxxxxx00
        // request:
        //   s: start bit = 1
        //   m: mode bit
        //   c: channel selection bit
        // response:
        //   r: response bit (msb first)
        //   x: checksum bit (lsb first)

        let start_bits = 1u32 << START_BIT_INDEX;
        let mode_bits = 0u32 << MODE_BIT_INDEX;
        let channel_selection_bits = (channel as u32) << CHANNEL_BITS_INDEX;
        (start_bits | mode_bits | channel_selection_bits).to_be_bytes()
        //[0xC0 | ((channel & 0x7) << 3), 0, 0]
    }

    // Configure the SPI peripheral. The 24AA1024 clocks in data on the first
    // rising edge of the clock signal (SPI mode 0). At 3.3 V, clock speeds of up
    // to 10 MHz are supported.
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0)?;

    let mut buffer = [0u8; 4];

    spi.transfer(&mut buffer, &create_write_buffer(0))?;

    println!("Bytes read: {:?}", buffer);

    Ok(())
}
