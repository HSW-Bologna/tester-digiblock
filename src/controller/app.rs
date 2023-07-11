use iced;
use iced::{Application, Command, Element};
use iced_native::widget::scrollable::{Id, RelativeOffset};
use std::fs;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::save_report;
use super::{flashing, worker};
use crate::controller::adc;
use crate::model::{
    Configuration, DigiblockState, Model, Report, RgbLight, StepState, TestState, TestStep,
    TestStepResult,
};
use crate::view;

const PORT: &str = "/dev/ttyACM0";
const CONFIG: &str = "./config.yaml";

#[derive(Clone, Debug)]
pub enum ControllerMessage {
    Connect(String),
    SetLight(RgbLight),
    Disconnect,
    Test(TestStep),
}

#[derive(Clone, Debug)]
pub enum ControllerEvent {
    Ready(mpsc::Sender<ControllerMessage>),
    Log(String),
    Update(DigiblockState),
    TestResult(TestStep, Option<f64>, bool),
}

#[derive(Clone, Debug)]
pub enum Event {
    UpdateLight(Instant),
    UpdateVBat(Instant),
    ViewEvent(view::Event),
    ControllerEvent(ControllerEvent),
}

pub struct App {
    model: Model,
    sender: Option<mpsc::Sender<ControllerMessage>>,
    start_ts: Instant,
}

impl Application for App {
    type Message = Event;
    type Theme = iced::theme::Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new((): Self::Flags) -> (App, Command<Event>) {
        let config = fs::read(CONFIG)
            .map(|v| {
                serde_yaml::from_str(String::from_utf8(v).unwrap_or("".into()).as_str())
                    .unwrap_or(Configuration::default())
            })
            .unwrap_or(Configuration::default());

        (
            App {
                model: Model {
                    config,
                    ..Model::default()
                },
                sender: None,
                start_ts: Instant::now(),
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
            Event::UpdateLight(_) => {
                let light = self.model.next_light();
                self.controller_message(ControllerMessage::SetLight(light));
                Command::none()
            }
            Event::UpdateVBat(_) => {
                self.model.add_vbat(worker::read_vbat().ok());
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::Update(state)) => {
                self.model.digiblock_update(state);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::TestResult(step, value, false)) => {
                self.add_test(step, false, value);
                self.model.state = TestState::Testing(step, StepState::Failed);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::TestResult(step, value, true)) => {
                self.add_test(step, true, value);
                self.next_step(step)
            }

            Event::ViewEvent(view::Event::UpdateDUT(val)) => {
                self.model.config.codice_dut = val;
                Command::none()
            }
            Event::ViewEvent(view::Event::UpdateOperator(val)) => {
                self.model.config.operatore = val.chars().take(20).collect();
                Command::none()
            }
            Event::ViewEvent(view::Event::Config) => {
                self.model.state = TestState::Unconfigured;
                Command::none()
            }
            Event::ViewEvent(view::Event::SaveConfig) => {
                fs::write(CONFIG, serde_yaml::to_string(&self.model.config).unwrap()).unwrap();
                self.model.state = TestState::Ready;
                Command::none()
            }

            Event::ViewEvent(view::Event::Start) => {
                self.model.report = Report::default();
                self.start_ts = Instant::now();
                self.model.state = TestState::Testing(TestStep::InvertPower, StepState::Waiting);
                Command::perform(worker::check_power_inversion(), |r| {
                    Event::ControllerEvent(ControllerEvent::TestResult(
                        TestStep::InvertPower,
                        None,
                        r,
                    ))
                })
            }
            Event::ViewEvent(view::Event::Retry) => {
                use TestStep::*;
                match self.model.state {
                    TestState::Testing(Connecting, _) => {
                        self.controller_message(ControllerMessage::Connect(PORT.into()));
                        Command::none()
                    }
                    TestState::Testing(InvertPower, _) => {
                        self.model.state =
                            TestState::Testing(TestStep::InvertPower, StepState::Waiting);
                        Command::perform(worker::check_power_inversion(), |r| {
                            Event::ControllerEvent(ControllerEvent::TestResult(
                                TestStep::InvertPower,
                                None,
                                r,
                            ))
                        })
                    }
                    TestState::Testing(step, _) => {
                        self.model.state = TestState::Testing(step, StepState::Waiting);
                        self.controller_message(ControllerMessage::Test(step));
                        Command::none()
                    }
                    _ => Command::none(),
                }
            }
            Event::ViewEvent(view::Event::UiOk) => match self.model.state {
                TestState::Testing(TestStep::UiLCD, _) => {
                    self.add_test(TestStep::UiLCD, true, None);
                    self.next_step(TestStep::UiLCD)
                }
                _ => {
                    self.add_test(TestStep::UiRgb, true, None);
                    self.next_step(TestStep::UiRgb)
                }
            },
            Event::ViewEvent(view::Event::Done) => {
                self.model.state = TestState::Ready;
                self.controller_message(ControllerMessage::Disconnect);
                save_report(&self.model);
                Command::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Event> {
        use iced::time::every;

        let mut subscriptions = vec![
            worker::worker().map(Event::ControllerEvent),
            every(Duration::from_millis(200)).map(Event::UpdateVBat),
        ];

        match self.model.state {
            TestState::Testing(TestStep::UiRgb, _) => {
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

    fn start_test(self: &mut Self, step: TestStep) -> Command<Event> {
        self.model.state = TestState::Testing(step, StepState::Waiting);
        self.controller_message(ControllerMessage::Test(step));
        Command::none()
    }

    fn test_power(self: &mut Self) -> Result<(), ()> {
        let power3v3 = adc::read_adc(adc::Channel::Volt3).map_err(|_| ())?;
        let power3v3 = ((power3v3 as f64 / 4095.0) * 3.35) * 2.0;
        let power3v3 = (power3v3 * 100.0).round() / 100.0;
        self.model
            .logs
            .push(format!("Tensione su linea 3v3: {}V", power3v3));

        if TestStep::Check3v3.check_limits(power3v3) {
            self.add_test(TestStep::Check3v3, true, Some(power3v3 as f64));
        } else {
            self.add_test(TestStep::Check3v3, false, Some(power3v3 as f64));
            return Err(());
        }

        let power5v = adc::read_adc(adc::Channel::Volt5).map_err(|_| ())?;
        let power5v = ((power5v as f64 / 4095.0) * 3.35) * 2.0;
        let power5v = (power5v * 100.0).round() / 100.0;

        self.model
            .logs
            .push(format!("Tensione su linea 5v: {}V", power5v));

        if TestStep::Check5v.check_limits(power5v) {
            self.add_test(TestStep::Check5v, true, Some(power5v as f64));
        } else {
            self.add_test(TestStep::Check5v, false, Some(power5v as f64));
            return Err(());
        }

        let power12v = adc::read_adc(adc::Channel::Supply).map_err(|_| ())?;
        let power12v = ((power12v as f64 / 4095.0) * 3.35) * (13.43 / 1.43);
        let power12v = (power12v * 100.0).round() / 100.0;

        self.model
            .logs
            .push(format!("Tensione su linea 12v: {}V", power12v));

        if TestStep::Check12v.check_limits(power12v) {
            self.add_test(TestStep::Check12v, true, Some(power12v as f64));
            Ok(())
        } else {
            self.add_test(TestStep::Check12v, false, Some(power12v as f64));
            Err(())
        }
    }

    fn next_step(&mut self, step: TestStep) -> Command<Event> {
        self.start_ts = Instant::now();

        use TestStep::*;
        match step {
            InvertPower => self.flash_test_firmware(),
            FlashingTest => {
                self.model.state = TestState::Testing(Connecting, StepState::Waiting);
                self.controller_message(ControllerMessage::Connect(String::from(PORT)));
                Command::none()
            }
            Connecting => {
                self.model.state = TestState::Testing(UiLeftButton, StepState::Waiting);
                Command::none()
            }
            UiLeftButton => {
                self.model.state = TestState::Testing(UiRightButton, StepState::Waiting);
                Command::none()
            }
            UiRightButton => {
                self.model.state = TestState::Testing(UiLCD, StepState::Waiting);
                Command::none()
            }
            UiLCD => {
                self.model.state = TestState::Testing(UiRgb, StepState::Waiting);
                self.model.light = RgbLight::default();
                Command::none()
            }
            UiRgb => {
                if self.test_power().is_ok() {
                    self.start_test(AnalogShortCircuit)
                } else {
                    Command::none()
                }
            }
            AnalogShortCircuit => self.start_test(Analog),
            Analog => self.start_test(Frequency),
            Frequency => self.start_test(OutputShortCircuit),
            OutputShortCircuit => self.start_test(Output),
            Output => self.flash_production_firmware(),
            FlashingProduction => {
                self.model.state = TestState::Done;
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn add_test(self: &mut Self, step: TestStep, result: bool, value: Option<f64>) {
        self.model.report.add_test(TestStepResult::new(
            step,
            result,
            value,
            Instant::now() - self.start_ts,
        ));
    }

    fn flash_test_firmware(self: &mut Self) -> Command<Event> {
        self.model.state = TestState::Testing(TestStep::FlashingTest, StepState::Waiting);
        Command::perform(flashing::load_test_firmware(), |res| {
            ControllerEvent::TestResult(
                TestStep::FlashingTest,
                None,
                if let Some(code) = res {
                    code == 0
                } else {
                    false
                },
            )
        })
        .map(Event::ControllerEvent)
    }

    fn flash_production_firmware(self: &mut Self) -> Command<Event> {
        self.model.state = TestState::Testing(TestStep::FlashingProduction, StepState::Waiting);
        Command::perform(flashing::load_production_firmware(), |res| {
            ControllerEvent::TestResult(
                TestStep::FlashingProduction,
                None,
                if let Some(code) = res {
                    code == 0
                } else {
                    false
                },
            )
        })
        .map(Event::ControllerEvent)
    }
}
