use iced;
use iced::{Application, Command, Element};
use iced_native::subscription::{self, Subscription};
use iced_native::widget::scrollable::{Id, RelativeOffset};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

use super::digiblock;
use super::flashing;
use crate::controller::reles::{self, Rele};
use crate::controller::{adc, pwm};
use crate::model::{DigiblockResult, DigiblockState, Model, RgbLight, TestStep};
use crate::view;

#[derive(Clone, Debug)]
pub enum ControllerMessage {
    Connect(String),
    SetLight(RgbLight),
    Disconnect,
    Frequency(u16),
    Pulses(u16),
    Analog,
    Output,
}

#[derive(Clone, Debug)]
pub enum ControllerEvent {
    Ready(mpsc::Sender<ControllerMessage>),
    Log(String),
    ConnectionFailed,
    Update(DigiblockState),
    PulsesDone(Result<u16, ()>),
    AnalogResult(DigiblockResult<u16>),
    OutputResult(DigiblockResult<()>),
}

#[derive(Clone, Debug)]
pub enum Event {
    UpdatePorts(Instant),
    UpdateLight(Instant),
    PowerInversionCheck(bool),
    Flashed(Option<i32>),
    ViewEvent(view::Event),
    ControllerEvent(ControllerEvent),
}

pub struct App {
    model: Model,
    sender: Option<mpsc::Sender<ControllerMessage>>,
}

impl Application for App {
    type Message = Event;
    type Theme = iced::theme::Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new((): Self::Flags) -> (App, Command<Event>) {
        (
            App {
                model: Model::default(),
                sender: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Digiblock Test")
    }

    fn view(&self) -> Element<Event> {
        view::view(&self.model).map(Event::ViewEvent)
    }

    fn update(&mut self, event: Event) -> Command<Event> {
        match event {
            Event::ControllerEvent(ControllerEvent::Ready(sender)) => {
                println!("Received message sender!");
                self.sender = Some(sender);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::Log(msg)) => {
                self.model.logs.push(msg);
                iced::widget::scrollable::snap_to(Id::new("logs"), RelativeOffset::END)
            }
            Event::UpdatePorts(_) => {
                //self.model.update_ports(get_ports());
                Command::none()
            }
            Event::UpdateLight(_) => {
                let light = self.model.next_light();
                self.controller_message(ControllerMessage::SetLight(light));
                Command::none()
            }
            Event::PowerInversionCheck(success) => {
                if success {
                    self.model.step = TestStep::FlashingTest(None);
                    Command::perform(flashing::load_test_firmware(), Event::Flashed)
                } else {
                    self.model.step = TestStep::InvertPower(Some(String::from("Errore")));
                    Command::none()
                }
            }
            Event::Flashed(result) => {
                match result {
                    Some(0) => {
                        self.model.step = TestStep::Connecting(None);
                        self.controller_message(ControllerMessage::Connect(String::from(
                            "/dev/ttyACM0",
                        )))
                    }
                    Some(code) => {
                        self.model.step =
                            TestStep::FlashingTest(Some(String::from(format!("{}", code))));
                    }
                    None => {
                        self.model.step = TestStep::FlashingTest(Some(String::from("Sconosciuto")));
                    }
                }
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::ConnectionFailed) => {
                self.model.step = TestStep::Connecting(Some(String::from("Error")));
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::Update(state)) => {
                self.model.digiblock_update(state);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::PulsesDone(res)) => {
                match self.model.step {
                    TestStep::Pulses(pulses, _) => {
                        self.model.step = TestStep::Pulses(pulses, Some(res))
                    }
                    _ => (),
                }
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::AnalogResult(res)) => {
                self.model.step = TestStep::Analog(res);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::OutputResult(res)) => {
                self.model.step = TestStep::Output(res);
                Command::none()
            }
            Event::ViewEvent(view::Event::Start) => {
                self.model.step = TestStep::InvertPower(None);
                Command::perform(check_power_inversion(), Event::PowerInversionCheck)
            }
            Event::ViewEvent(view::Event::Retry) => {
                match self.model.step {
                    TestStep::Connecting(_) => self.controller_message(ControllerMessage::Connect(
                        String::from("/dev/ttyACM0"),
                    )),
                    _ => (),
                };
                Command::none()
            }
            Event::ViewEvent(view::Event::Next) => {
                match self.model.step {
                    TestStep::Ui { light } => {
                        let _ = light;
                        self.model.step = TestStep::Frequency(1000, false);
                        self.controller_message(ControllerMessage::Frequency(1000));
                    }
                    TestStep::Frequency(_, _) => {
                        self.model.step = TestStep::Pulses(1000, None);
                        self.controller_message(ControllerMessage::Pulses(1000));
                    }
                    TestStep::Pulses(_, _) => {
                        self.model.step = TestStep::Analog(DigiblockResult::Waiting);
                        self.controller_message(ControllerMessage::Analog);
                    }
                    TestStep::Analog(_) => {
                        self.model.step = TestStep::Output(DigiblockResult::Waiting);
                        self.controller_message(ControllerMessage::Output);
                    }
                    _ => (),
                }
                Command::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Event> {
        use iced::time::every;

        let mut subscriptions = vec![
            controller_worker().map(Event::ControllerEvent),
            every(Duration::from_millis(1000)).map(Event::UpdatePorts),
        ];

        match self.model.step {
            TestStep::Ui { light } => {
                let _ = light;
                subscriptions.push(every(Duration::from_millis(1000)).map(Event::UpdateLight));
            }
            _ => (),
        }

        iced::Subscription::batch(subscriptions)
    }

    fn theme(&self) -> iced::theme::Theme {
        iced::theme::Theme::Dark
    }
}

impl App {
    fn controller_message(&self, msg: ControllerMessage) {
        if let Some(sender) = self.sender.clone() {
            tokio::spawn(async move { sender.send(msg).await });
        }
    }
}

fn controller_worker() -> Subscription<ControllerEvent> {
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
                                            log(&mut output, "Connection successful").await;
                                            state = State::Connected(ctx);
                                            timestamp = Instant::now();
                                        } else {
                                            log(&mut output, "Connection failed").await;
                                            output
                                                .send(ControllerEvent::ConnectionFailed)
                                                .await
                                                .ok();
                                        }
                                    } else {
                                        log(&mut output, "Connection failed").await;
                                        output.send(ControllerEvent::ConnectionFailed).await.ok();
                                    }
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
                                ControllerMessage::Frequency(frequency) => {
                                    //TODO: set error
                                    start_frequency(ctx, frequency).await.ok();
                                }
                                ControllerMessage::Pulses(pulses) => {
                                    output
                                        .send(ControllerEvent::PulsesDone(
                                            check_pulses(ctx, pulses).await,
                                        ))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Analog => {
                                    output
                                        .send(ControllerEvent::AnalogResult(
                                            match check_analog(ctx).await {
                                                Ok(None) => DigiblockResult::Ok,
                                                Ok(Some((expected, found))) => {
                                                    DigiblockResult::InvalidValue(expected, found)
                                                }
                                                Err(()) => DigiblockResult::CommunicationError,
                                            },
                                        ))
                                        .await
                                        .ok();
                                }
                                ControllerMessage::Output => {
                                    output
                                        .send(ControllerEvent::OutputResult(
                                            match check_output(ctx).await {
                                                Ok(true) => DigiblockResult::Ok,
                                                Ok(false) => DigiblockResult::InvalidValue((), ()),
                                                Err(()) => DigiblockResult::CommunicationError,
                                            },
                                        ))
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
                                    output.send(ControllerEvent::ConnectionFailed).await.ok();
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

async fn check_power_inversion() -> bool {
    sleep(Duration::from_millis(500)).await;

    //TODO: read adc channel
    true
}

async fn check_pulses(ctx: &mut Context, pulses: u16) -> Result<u16, ()> {
    reles::update(Rele::AnalogMode, false).map_err(|_| ())?;
    reles::update(Rele::DigitalMode, true).map_err(|_| ())?;

    digiblock::set_frequency_mode(ctx).await.map_err(|_| ())?;
    digiblock::reset_pulses(ctx).await.map_err(|_| ())?;
    pwm::toggle_times(pulses).await.map_err(|_| ())?;

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
    sleep(Duration::from_millis(100)).await;
    let value = adc::read_adc(adc::Channel::Out1).map_err(|_| ())?;
    println!("adc {}", value);

    digiblock::set_output(ctx, true).await.map_err(|_| ())?;
    sleep(Duration::from_millis(100)).await;
    let value = adc::read_adc(adc::Channel::Out1).map_err(|_| ())?;
    println!("adc {}", value);

    Ok(true)
}

async fn start_frequency(ctx: &mut Context, frequency: u16) -> Result<(), ()> {
    reles::update(Rele::DigitalMode, true).map_err(|_| ())?;
    digiblock::set_frequency_mode(ctx).await.map_err(|_| ())?;
    pwm::set_frequency(frequency).map_err(|_| ())?;

    Ok(())
}
