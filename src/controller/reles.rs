use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;

#[derive(Clone, Copy, Debug)]
pub enum Rele {
    Enable420ma,
    ShortCircuit,
}

pub fn update(rele: Rele, value: bool) -> Result<(), Box<dyn Error>> {
    let gpio = match rele {
        Rele::ShortCircuit => 2,
        Rele::Enable420ma => 3,
    };

    let mut pin = Gpio::new()?.get(gpio)?.into_output();
    pin.set_reset_on_drop(false);

    /*pin.set_high();
    thread::sleep(Duration::from_millis(500));
    pin.set_low();
    thread::sleep(Duration::from_millis(500));
    pin.set_high();
    thread::sleep(Duration::from_millis(500));
    pin.set_low();
    thread::sleep(Duration::from_millis(500));
    */

    println!("Setting pin {:?} to {}", rele, value);

    if value {
        pin.set_high();
    } else {
        pin.set_low();
    }

    Ok(())
}

pub fn set_reles() -> Result<(), Box<dyn Error>> {
    println!("Blinking an LED on a {}.", DeviceInfo::new()?.model());

    for x in [25, 2, 3, 4, 17, 27, 22, 14] {
        println!("Working with {}", x);
        let mut pin = Gpio::new()?.get(x)?.into_output();

        // Blink the LED by setting the pin's logic level high for 500 ms.
        pin.set_high();
        thread::sleep(Duration::from_millis(500));
        pin.set_low();
        thread::sleep(Duration::from_millis(500));
        pin.set_high();
        thread::sleep(Duration::from_millis(500));
        pin.set_low();
        thread::sleep(Duration::from_millis(2000));
    }

    Ok(())
}
