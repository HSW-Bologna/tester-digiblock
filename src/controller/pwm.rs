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

pub fn set_frequency(frequency: u16) -> Result<(), ()> {
    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 2 Hz with a 25% duty cycle.
    let mut pwm = Pwm::with_frequency(Channel::Pwm0, frequency as f64, 0.5, Polarity::Normal, true)
        .map_err(|_| ())?;
    pwm.set_reset_on_drop(false);

    Ok(())
}

pub async fn toggle_times(times: u16) -> Result<(), Box<dyn Error>> {
    for _ in 0..times {
        set_pulse_level(false)?;
        tokio::time::sleep(Duration::from_micros(500)).await;
        set_pulse_level(true)?;
        tokio::time::sleep(Duration::from_micros(500)).await;
    }

    set_pulse_level(false)?;

    Ok(())
}

fn set_pulse_level(value: bool) -> Result<(), Box<dyn Error>> {
    let mut pin = Gpio::new()?.get(12)?.into_output();
    pin.set_reset_on_drop(false);

    if value {
        pin.set_high();
    } else {
        pin.set_low();
    }

    Ok(())
}
