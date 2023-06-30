use iced_native::{subscription, Subscription};
use tokio_serial::SerialStream;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_modbus::{client::Context, slave::Slave};
use tokio::time::sleep;

use crate::{controller::{
    app::{ControllerEvent, ControllerMessage},
    reles::{self, Rele}, digiblock, adc,
}, model::TestStep};

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
                                    println!("Connect to {}", port);

                                    reles::update(Rele::IncorrectPower, false).ok();
                                    reles::update(Rele::CorrectPower, true).ok();
                                    sleep(Duration::from_millis(500)).await;

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

                                    if let State::Connected(_) = state {
                                        log(&mut output, "Connessione effettuata").await;
                                        output
                                            .send(ControllerEvent::TestResult(
                                                TestStep::Connecting,
                                                true,
                                            ))
                                            .await
                                            .ok();
                                    } else {
                                        log(&mut output, "Connessione fallita").await;
                                        output
                                            .send(ControllerEvent::TestResult(
                                                TestStep::Connecting,
                                                false,
                                            ))
                                            .await
                                            .ok();
                                    }
                                }
                                // Not connected, fail
                                ControllerMessage::Test(step) => {
                                    output
                                        .send(ControllerEvent::TestResult(step, false))
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
                            println!("Received msg {:?}", msg);
                            match msg {
                                ControllerMessage::Disconnect => {
                                    ctx.disconnect().await.ok();
                                    state = State::Disconnected;
                                }
                                ControllerMessage::SetLight(light) => {
                                    digiblock::set_light(ctx, light).await.ok();
                                }
                                ControllerMessage::Test(TestStep::Frequency) => {
                                    let frequency = 1000;
                                    let res = match check_frequency(ctx, frequency).await {
                                        Ok(found) => {
                                            if i32::abs((frequency as i32) - (found as i32)) > 1 {
                                                log(
                                                    &mut output,
                                                    format!(
                                                        "Frequenza non corrispondente: {} - {}",
                                                        frequency, found
                                                    ),
                                                )
                                                .await;
                                                false
                                            } else {
                                                true
                                            }
                                        }
                                        Err(()) => {
                                            log(
                                                &mut output,
                                                "Errore nell'impostazione della frequenza",
                                            )
                                            .await;
                                            false
                                        }
                                    };

                                    output
                                        .send(ControllerEvent::TestResult(TestStep::Frequency, res))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Test(TestStep::Pulses) => {
                                    let pulses = 1000;
                                    if let Ok(found) = check_pulses(ctx, pulses).await {
                                        log(
                                            &mut output,
                                            format!("Impulsi: {} - {}", pulses, found),
                                        )
                                        .await;

                                        output
                                            .send(ControllerEvent::TestResult(
                                                TestStep::Pulses,
                                                true,
                                            ))
                                            .await
                                            .ok();
                                    } else {
                                        println!("Error pulses");
                                        log(&mut output, format!("Errore nell'invio di impulsi"))
                                            .await;
                                        output
                                            .send(ControllerEvent::TestResult(
                                                TestStep::Pulses,
                                                false,
                                            ))
                                            .await
                                            .ok();
                                    }
                                }
                                ControllerMessage::Test(TestStep::Analog) => {
                                    let res = match check_analog(ctx).await {
                                        Ok(None) => true,
                                        Ok(Some((expected, found))) => {
                                            log(
                                                &mut output,
                                                format!("Analogico: {} - {}", expected, found),
                                            )
                                            .await;
                                            false
                                        }
                                        Err(()) => false,
                                    };

                                    output
                                        .send(ControllerEvent::TestResult(TestStep::Analog, res))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Test(TestStep::Output) => {
                                    let res = match check_output(ctx).await {
                                        Ok(true) => true,
                                        Ok(false) => false,
                                        Err(()) => false,
                                    };

                                    output
                                        .send(ControllerEvent::TestResult(TestStep::Output, res))
                                        .await
                                        .ok();
                                }
                                // Not implemented, fail
                                ControllerMessage::Test(step) => {
                                    output
                                        .send(ControllerEvent::TestResult(step, false))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Connect(_) => (),
                            }
                        } else {
                            let now = Instant::now();

                            if now.duration_since(timestamp) > Duration::from_millis(500) {
                                timestamp = now;

                                if let Ok(Ok(rsp)) =
                                    timeout(Duration::from_millis(50), digiblock::get_state(ctx))
                                        .await
                                {
                                    output.send(ControllerEvent::Update(rsp)).await.ok();
                                } else {
                                    log(&mut output, "Errore di comunicazione").await;
                                    output
                                        .send(ControllerEvent::TestResult(
                                            TestStep::Connecting,
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

pub async fn check_power_inversion() -> bool {
    sleep(Duration::from_millis(500)).await;

    //TODO: read adc channel
    true
}

async fn check_pulses(ctx: &mut Context, pulses: u16) -> Result<u16, ()> {
    reles::update(Rele::AnalogMode, false).map_err(|_| ())?;
    reles::update(Rele::DigitalMode, true).map_err(|_| ())?;

    println!("Setting freq");
    digiblock::set_frequency_mode(ctx).await.map_err(|_| ())?;
    println!("Reset pulses");
    digiblock::reset_pulses(ctx).await.map_err(|_| ())?;

    println!("Sending {} pulses", pulses);
    pwm::toggle_times(pulses).await.map_err(|_| ())?;

    println!("Pulses sent");
    let rsp = tokio::time::timeout(Duration::from_millis(50), digiblock::get_state(ctx))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

    Ok(rsp.pulses)
}

async fn check_analog(ctx: &mut Context) -> Result<Option<(u16, u16)>, ()> {
    reles::update(Rele::DigitalMode, false).map_err(|_| ())?;
    reles::update(Rele::AnalogMode, true).map_err(|_| ())?;

    digiblock::set_analog_mode(ctx).await.map_err(|_| ())?;

    pwm::set_420ma(2).map_err(|_| ())?;
    sleep(Duration::from_millis(500)).await;

    let rsp = tokio::time::timeout(Duration::from_millis(50), digiblock::get_state(ctx))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

    println!("{:?}", rsp);

    Ok(None)
}

async fn check_output(ctx: &mut Context) -> Result<bool, ()> {
    digiblock::set_output(ctx, false).await.map_err(|_| ())?;
    sleep(Duration::from_millis(5000)).await;
    let value = adc::read_adc(adc::Channel::Out1).map_err(|_| ())?;
    println!("adc {}", value);

    digiblock::set_output(ctx, true).await.map_err(|_| ())?;
    sleep(Duration::from_millis(100)).await;
    let value = adc::read_adc(adc::Channel::Out1).map_err(|_| ())?;
    println!("adc {}", value);

    Ok(true)
}

async fn check_frequency(ctx: &mut Context, frequency: u16) -> Result<u16, ()> {
    reles::update(Rele::DigitalMode, true).map_err(|_| ())?;
    digiblock::set_frequency_mode(ctx).await.map_err(|_| ())?;
    pwm::set_frequency(frequency).map_err(|_| ())?;

    sleep(Duration::from_millis(100)).await;

    let rsp = tokio::time::timeout(Duration::from_millis(50), digiblock::get_state(ctx))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())?;

    Ok(rsp.frequency)
}
