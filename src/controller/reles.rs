use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;

#[derive(Clone, Copy, Debug)]
pub enum Rele {
    ShortCircuitOutput,
    ShortCircuitAnalog,
    CorrectPower,
    IncorrectPower,
    UsbGround,
    DigitalMode,
    AnalogMode,
}

pub fn update(rele: Rele, value: bool) -> Result<(), ()> {
    let gpio = match rele {
        Rele::ShortCircuitOutput => 14,
        Rele::ShortCircuitAnalog => 2,
        Rele::AnalogMode => 3, // 420ma
        Rele::CorrectPower => 4,
        Rele::IncorrectPower => 17,
        Rele::UsbGround => 25,
        Rele::DigitalMode => 22, // Frequency
    };

    if value {
        match rele {
            Rele::CorrectPower => {
                let mut pin = Gpio::new()
                    .map_err(|_| ())?
                    .get(17)
                    .map_err(|_| ())?
                    .into_output();
                pin.set_reset_on_drop(false);
                pin.set_low();
            }
            Rele::IncorrectPower => {
                let mut pin = Gpio::new()
                    .map_err(|_| ())?
                    .get(4)
                    .map_err(|_| ())?
                    .into_output();
                pin.set_reset_on_drop(false);
                pin.set_low();
            }
            Rele::DigitalMode => {
                let mut pin = Gpio::new()
                    .map_err(|_| ())?
                    .get(3)
                    .map_err(|_| ())?
                    .into_output();
                pin.set_reset_on_drop(false);
                pin.set_low();
            }
            Rele::AnalogMode => {
                let mut pin = Gpio::new()
                    .map_err(|_| ())?
                    .get(22)
                    .map_err(|_| ())?
                    .into_output();
                pin.set_reset_on_drop(false);
                pin.set_low();
            }
            _ => (),
        }
    }

    let mut pin = Gpio::new()
        .map_err(|_| ())?
        .get(gpio)
        .map_err(|_| ())?
        .into_output();
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
    update(Rele::ShortCircuitOutput, false).ok();
    update(Rele::ShortCircuitAnalog, false).ok();
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
