use std::{error::Error, time::Duration};

use rppal::{
    gpio::Gpio,
    pwm::{Channel, Polarity, Pwm},
};

pub fn set_420ma(milliamperes: i32) -> Result<(), Box<dyn Error>> {
    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 2 Hz with a 25% duty cycle.

    let milliamperes: f64 = 20.0 - milliamperes as f64;
    let _percentage = milliamperes / 16.0;

    let mut pwm = Pwm::with_frequency(Channel::Pwm1, 1000.0, 1.0 - 0.394, Polarity::Normal, true)?;
    pwm.set_reset_on_drop(false);

    Ok(())
}

pub async fn set_frequency(frequency: u16) -> Result<(), ()> {
    /*let mut pin = Gpio::new().unwrap().get(12).unwrap().into_output();
    pin.set_reset_on_drop(false);
    let period = 1000_000 / frequency as u64;
    pin.set_pwm(Duration::from_micros(period), Duration::from_micros(period/2)).unwrap();
    Ok(())*/

    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 2 Hz with a 25% duty cycle.
    let frequency = (frequency as f64) * 0.932; // * 0.95;

    let mut pwm = Pwm::with_frequency(Channel::Pwm0, frequency, 0.5, Polarity::Normal, true)
        .map_err(|_| ())?;
    pwm.set_reset_on_drop(false);
    Ok(())

    // Need to use pigs, for some reason rppal is very inaccurate

    /*
    use tokio::process::Command;
         Command::new("pigs")
        .args(&["hp", "12", format!("{}", frequency).as_str(), "500000"])
        .status()
        .await
        .map(|_| ())
        .map_err(|_| ())
        */
}

pub async fn toggle_times(times: u16) -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(12)?.into_output();

    for _ in 0..times {
        pin.set_low();
        tokio::time::sleep(Duration::from_micros(50)).await;
        pin.set_high();
        tokio::time::sleep(Duration::from_micros(50)).await;
    }

    pin.set_low();

    Ok(())
}
