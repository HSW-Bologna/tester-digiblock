use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;

#[derive(Clone, Copy, Debug)]
pub enum Rele {
    ShortCircuit,
    CorrectPower,
    IncorrectPower,
    UsbGround,
    DigitalMode,
    AnalogMode,
}

pub fn update(rele: Rele, value: bool) -> Result<(), Box<dyn Error>> {
    let gpio = match rele {
        Rele::ShortCircuit => 2,
        Rele::AnalogMode => 3, // 420ma
        Rele::CorrectPower => 4,
        Rele::IncorrectPower => 17,
        Rele::UsbGround => 25,
        Rele::DigitalMode => 22, // Frequency
    };

    match rele {
        Rele::CorrectPower => {
            let mut pin = Gpio::new()?.get(17)?.into_output();
            pin.set_reset_on_drop(false);
            pin.set_low();
        }
        Rele::IncorrectPower => {
            let mut pin = Gpio::new()?.get(4)?.into_output();
            pin.set_reset_on_drop(false);
            pin.set_low();
        }
        Rele::DigitalMode => {
            let mut pin = Gpio::new()?.get(3)?.into_output();
            pin.set_reset_on_drop(false);
            pin.set_low();
        }
        Rele::AnalogMode => {
            let mut pin = Gpio::new()?.get(22)?.into_output();
            pin.set_reset_on_drop(false);
            pin.set_low();
        }
        _ => (),
    }

    let mut pin = Gpio::new()?.get(gpio)?.into_output();
    pin.set_reset_on_drop(false);

    println!("Setting pin {:?} to {}", rele, value);

    if value {
        pin.set_high();
    } else {
        pin.set_low();
    }

    Ok(())
}

pub fn all_off() {
    update(Rele::AnalogMode, false).ok();
    update(Rele::ShortCircuit, false).ok();
    update(Rele::CorrectPower, false).ok();
    update(Rele::IncorrectPower, false).ok();
    update(Rele::UsbGround, false).ok();
    update(Rele::DigitalMode, false).ok();
}

pub fn _set_reles() -> Result<(), Box<dyn Error>> {
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
