use std::{error::Error, time::Duration};

use rppal::{
    gpio::Gpio,
    pwm::{Channel, Polarity, Pwm},
};

pub fn set_420ma(_milliamperes: u16) -> Result<(), Box<dyn Error>> {
    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 2 Hz with a 25% duty cycle.
    let mut pwm = Pwm::with_frequency(Channel::Pwm1, 1000.0, 0.25, Polarity::Normal, true)?;
    pwm.set_reset_on_drop(false);

    Ok(())
}

pub fn set_frequency(frequency: u16) -> Result<(), Box<dyn Error>> {

    /*let mut pin = Gpio::new()?.get(12)?.into_output();
    pin.set_reset_on_drop(false);

    pin.set_pwm(Duration::from_millis(10), Duration::from_millis(5))?;
    */

    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 2 Hz with a 25% duty cycle.
    let mut pwm = Pwm::with_frequency(Channel::Pwm0, frequency as f64, 0.5, Polarity::Normal, true)?;
    pwm.set_reset_on_drop(false);

    Ok(())
}

pub async fn toggle_times(times: u16) -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(12)?.into_output();

    for _ in 0..times {
        pin.set_low();
        tokio::time::sleep(Duration::from_micros(500)).await;
        pin.set_high();
        tokio::time::sleep(Duration::from_micros(500)).await;
    }

    pin.set_low();

    Ok(())
}
