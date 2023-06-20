
use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::pwm::{Channel, Polarity, Pwm};

pub fn set_pwm() -> Result<(), Box<dyn Error>> {
    // Enable PWM channel 0 (BCM GPIO 18, physical pin 12) at 2 Hz with a 25% duty cycle.
    let mut pwm = Pwm::with_frequency(Channel::Pwm1, 1000.0, 0.25, Polarity::Normal, true)?;
    pwm.set_reset_on_drop(false);

    println!("Running pwm1 at 25%");
    thread::sleep(Duration::from_secs(10));

    Ok(())

    // When the pwm variable goes out of scope, the PWM channel is automatically disabled.
    // You can manually disable the channel by calling the Pwm::disable() method.
}
