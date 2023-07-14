use iced_native::{subscription, Subscription};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_modbus::{client::Context, slave::Slave};
use tokio_serial::SerialStream;

use crate::{
    controller::{
        adc,
        app::{ControllerEvent, ControllerMessage},
        digiblock,
        reles::{self, Rele},
    },
    model::TestStep,
};

use super::pwm;

pub fn worker() -> Subscription<ControllerEvent> {
    struct SomeWorker;

    async fn log(
        sender: &mut iced_futures::futures::channel::mpsc::Sender<ControllerEvent>,
        msg: impl Into<String> + std::fmt::Display,
    ) {
        use chrono::prelude::*;

        let now = Utc::now();
        let time = now.format("%H:%M:%S:%3f");

        sender
            .send(ControllerEvent::Log(format!("[{}]: {}", time, msg)))
            .await
            .ok();
    }

    async fn frequency_test(
        ctx: &mut Context,
        output: &mut iced_futures::futures::channel::mpsc::Sender<ControllerEvent>,
        step: TestStep,
        frequency: u16,
    ) {
        let (res, value) = match check_frequency(ctx, frequency).await {
            Ok(found) => (check_value_within(found, step.limits()), Some(found)),
            Err(()) => (false, None),
        };

        output
            .send(ControllerEvent::TestResult(step, value, res))
            .await
            .ok();
    }

    async fn analog_test(
        ctx: &mut Context,
        output: &mut iced_futures::futures::channel::mpsc::Sender<ControllerEvent>,
        step: TestStep,
        ma420: i32,
    ) {
        let (res, value) = match check_analog(ctx, ma420).await {
            Ok(found) => (check_value_within(found, step.limits()), Some(found)),
            Err(()) => (false, None),
        };

        output
            .send(ControllerEvent::TestResult(step, value, res))
            .await
            .ok();
    }

    use iced::futures::SinkExt;
    use tokio::time::timeout;

    enum State {
        Disconnected,
        Connected(Context),
    }

    subscription::channel(
        std::any::TypeId::of::<SomeWorker>(),
        32,
        |mut output| async move {
            let mut state = State::Disconnected;
            let mut timestamp = Instant::now();

            let (sender, mut receiver) = mpsc::channel(32);
            output.send(ControllerEvent::Ready(sender)).await.ok();

            loop {
                match &mut state {
                    State::Disconnected => {
                        if let Ok(Some(msg)) =
                            timeout(Duration::from_millis(1000), receiver.recv()).await
                        {
                            match msg {
                                ControllerMessage::Connect(port) => {
                                    reset().await;

                                    let builder = tokio_serial::new(port, 115200);
                                    if let Ok(port) = SerialStream::open(&builder) {
                                        if let Ok(ctx) = tokio_modbus::client::rtu::connect_slave(
                                            port,
                                            Slave(0x01),
                                        )
                                        .await
                                        {
                                            state = State::Connected(ctx);
                                            timestamp = Instant::now();
                                        }
                                    }

                                    output
                                        .send(ControllerEvent::TestResult(
                                            TestStep::Connecting,
                                            None,
                                            if let State::Connected(_) = state {
                                                true
                                            } else {
                                                false
                                            },
                                        ))
                                        .await
                                        .ok();
                                }
                                // Not connected, fail
                                ControllerMessage::Test(step) => {
                                    output
                                        .send(ControllerEvent::TestResult(step, None, false))
                                        .await
                                        .ok();
                                }
                                _ => (),
                            }
                        }
                    }
                    State::Connected(ref mut ctx) => {
                        if let Ok(Some(msg)) =
                            timeout(Duration::from_millis(100), receiver.recv()).await
                        {
                            match msg {
                                ControllerMessage::Disconnect => {
                                    ctx.disconnect().await.ok();
                                    state = State::Disconnected;
                                }
                                ControllerMessage::SetLight(light) => {
                                    digiblock::set_light(ctx, light).await.ok();
                                }
                                ControllerMessage::Test(TestStep::AnalogShortCircuit) => {
                                    let res =
                                        check_analog_short_circuit(ctx).await.unwrap_or(false);

                                    output
                                        .send(ControllerEvent::TestResult(
                                            TestStep::AnalogShortCircuit,
                                            None,
                                            res,
                                        ))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Test(TestStep::Analog) => {
                                    analog_test(ctx, &mut output, TestStep::Analog, 10).await;
                                }
                                ControllerMessage::Test(TestStep::Frequency) => {
                                    frequency_test(ctx, &mut output, TestStep::Frequency, 2000)
                                        .await;
                                }
                                ControllerMessage::Test(TestStep::OutputShortCircuit) => {
                                    let res =
                                        check_output_short_circuit(ctx).await.unwrap_or(false);

                                    output
                                        .send(ControllerEvent::TestResult(
                                            TestStep::OutputShortCircuit,
                                            None,
                                            res,
                                        ))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Test(TestStep::Output) => {
                                    let res = check_output(ctx).await.unwrap_or(false);

                                    output
                                        .send(ControllerEvent::TestResult(
                                            TestStep::Output,
                                            None,
                                            res,
                                        ))
                                        .await
                                        .ok();
                                }
                                // Not implemented, fail
                                ControllerMessage::Test(step) => {
                                    output
                                        .send(ControllerEvent::TestResult(step, None, false))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Connect(_) => (),
                            }
                        } else {
                            let now = Instant::now();

                            if now.duration_since(timestamp) > Duration::from_millis(250) {
                                timestamp = now;

                                let result =
                                    timeout(Duration::from_millis(50), digiblock::get_state(ctx))
                                        .await;
                                if let Ok(Ok(rsp)) = result {
                                    output.send(ControllerEvent::Update(rsp)).await.ok();
                                } else {
                                    log(&mut output, "Errore di comunicazione").await;
                                    output
                                        .send(ControllerEvent::TestResult(
                                            TestStep::Connecting,
                                            None,
                                            false,
                                        ))
                                        .await
                                        .ok();
                                    state = State::Disconnected;
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

fn _get_ports() -> Vec<String> {
    serialport::available_ports()
        .unwrap_or(Vec::new())
        .iter()
        .map(|p| p.port_name.clone())
        .collect()
}

pub async fn check_power_inversion() -> Result<f64, ()> {
    reles::update(reles::Rele::UsbGround, true).ok();
    sleep(Duration::from_millis(50)).await;
    reles::update(reles::Rele::IncorrectPower, true).ok();
    sleep(Duration::from_millis(500)).await;
    let result = adc::read_adc(adc::Channel::PowerConsumption);

    reles::update(reles::Rele::UsbGround, false).ok();
    sleep(Duration::from_millis(50)).await;
    reles::update(reles::Rele::IncorrectPower, false).ok();
    sleep(Duration::from_millis(500)).await;

    if let Ok(power) = result {
        Ok(power as f64)
    } else {
        Err(())
    }
}

pub fn read_vbat() -> Result<f64, ()> {
    let res = adc::read_adc(adc::Channel::VBat).map_err(|_| ())?;
    Ok(res as f64)
}

async fn _check_pulses(ctx: &mut Context, pulses: u16) -> Result<u16, ()> {
    reles::update(Rele::AnalogMode, false).map_err(|_| ())?;
    reles::update(Rele::DigitalMode, true).map_err(|_| ())?;

    digiblock::set_frequency_mode(ctx).await.map_err(|_| ())?;

    let mut counter = 0;
    let result = loop {
        counter += 1;

        if counter > 5 {
            break false;
        }

        if let Ok(_) =
            tokio::time::timeout(Duration::from_millis(50), digiblock::_reset_pulses(ctx)).await
        {
            break true;
        } else {
        }
    };

    if !result {
        return Err(());
    }

    pwm::toggle_times(pulses).await.map_err(|_| ())?;

    let rsp = tokio::time::timeout(Duration::from_millis(50), digiblock::get_state(ctx))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

    Ok(rsp.pulses)
}

async fn check_analog_short_circuit(ctx: &mut Context) -> Result<bool, ()> {
    reles::update(Rele::DigitalMode, false)?;
    reles::update(Rele::AnalogMode, true)?;
    reles::update(Rele::ShortCircuitAnalog, false)?;
    digiblock::set_analog_mode(ctx).await?;
    sleep(Duration::from_millis(200)).await;

    let short_circuit = digiblock::get_short_circuit_adc(ctx).await?;
    if short_circuit {
        return Ok(false);
    }

    reles::update(Rele::ShortCircuitAnalog, true)?;
    sleep(Duration::from_millis(200)).await;

    let short_circuit = digiblock::get_short_circuit_adc(ctx).await?;

    reles::update(Rele::ShortCircuitAnalog, false)?;

    Ok(short_circuit)
}

async fn check_analog(ctx: &mut Context, ma420: i32) -> Result<f64, ()> {
    reles::update(Rele::ShortCircuitAnalog, false)?;
    reles::update(Rele::DigitalMode, false)?;
    reles::update(Rele::AnalogMode, true)?;

    digiblock::set_analog_mode(ctx).await?;

    pwm::set_420ma(ma420).map_err(|_| ())?;
    sleep(Duration::from_millis(500)).await;

    let rsp = tokio::time::timeout(Duration::from_millis(50), digiblock::get_state(ctx))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

    let resulting_420ma = (rsp.ma420 as f64) / 100.0;

    Ok(resulting_420ma)
}

async fn check_output_short_circuit(ctx: &mut Context) -> Result<bool, ()> {
    // Toggling short circuit
    reles::update(reles::Rele::ShortCircuitOutput, false).map_err(|_| ())?;
    digiblock::set_output(ctx, true).await.map_err(|_| ())?;
    sleep(Duration::from_millis(200)).await;

    let short_circuit = digiblock::get_short_circuit_out(ctx).await?;
    if short_circuit {
        return Ok(false);
    }

    reles::update(reles::Rele::ShortCircuitOutput, true).map_err(|_| ())?;
    sleep(Duration::from_millis(200)).await;

    let short_circuit = digiblock::get_short_circuit_out(ctx).await?;

    reles::update(reles::Rele::ShortCircuitOutput, false).map_err(|_| ())?;

    Ok(short_circuit)
}

async fn check_output(ctx: &mut Context) -> Result<bool, ()> {
    reles::update(reles::Rele::ShortCircuitOutput, false).map_err(|_| ())?;
    digiblock::set_output(ctx, false).await.map_err(|_| ())?;
    sleep(Duration::from_millis(500)).await;

    toggle_output(ctx).await
}

async fn toggle_output(ctx: &mut Context) -> Result<bool, ()> {
    digiblock::set_output(ctx, true).await.map_err(|_| ())?;
    sleep(Duration::from_millis(500)).await;
    let value = adc::read_adc(adc::Channel::Out1).map_err(|_| ())?;

    if value < 1000 {
        return Ok(false);
    }

    digiblock::set_output(ctx, false).await.map_err(|_| ())?;
    sleep(Duration::from_millis(100)).await;
    let value = adc::read_adc(adc::Channel::Out1).map_err(|_| ())?;

    if value > 1000 {
        Ok(false)
    } else {
        Ok(true)
    }
}

async fn check_frequency(ctx: &mut Context, frequency: u16) -> Result<f64, ()> {
    reles::update(Rele::DigitalMode, true).map_err(|_| ())?;
    digiblock::set_frequency_mode(ctx).await.map_err(|_| ())?;
    pwm::set_frequency(frequency).await.map_err(|_| ())?;
    sleep(Duration::from_millis(500)).await;

    let rsp = tokio::time::timeout(Duration::from_millis(50), digiblock::get_state(ctx))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

    let found: f64 = if rsp.period_us == 0 {
        0.0
    } else {
        1_000_000.0 / (rsp.period_us as f64)
    };


    pwm::toggle_times(0).await.map_err(|_| ())?;

    Ok(found)
}

fn check_value_within<T>(found: T, limits: Option<(T, T)>) -> bool
where
    T: PartialOrd
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + Copy
        + core::fmt::Display,
{
    if let Some((min, max)) = limits {
        found >= min && found <= max
    } else {
        false
    }
}

pub async fn reset() {
    reles::update(Rele::IncorrectPower, false).ok();
    reles::update(Rele::CorrectPower, false).ok();
    sleep(Duration::from_millis(500)).await;
    reles::update(Rele::CorrectPower, true).ok();
    sleep(Duration::from_millis(1000)).await;
}
