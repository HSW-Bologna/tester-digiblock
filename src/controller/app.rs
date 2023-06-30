use iced;
use iced::{Application, Command, Element};
use iced_native::widget::scrollable::{Id, RelativeOffset};
use tokio_modbus::client::Context;
use std::fs;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::{flashing, worker};
use super::save_report;
use crate::model::{
    Configuration, DigiblockState, Model, Report, RgbLight, StepState, TestState, TestStep,
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
    TestResult(TestStep, bool),
}

#[derive(Clone, Debug)]
pub enum Event {
    UpdateLight(Instant),
    PowerInversionCheck(bool),
    Flashed(Option<i32>),
    ViewEvent(view::Event),
    ControllerEvent(ControllerEvent),
}

pub struct App {
    model: Model,
    sender: Option<mpsc::Sender<ControllerMessage>>,
    //connection: Option<Context>;
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
            Event::PowerInversionCheck(success) => {
                if success {
                    self.model.state =
                        TestState::Testing(TestStep::FlashingTest, StepState::Waiting);
                    Command::perform(flashing::load_test_firmware(), Event::Flashed)
                } else {
                    self.model.state = TestState::Testing(TestStep::InvertPower, StepState::Failed);
                    Command::none()
                }
            }
            Event::Flashed(result) => {
                match result {
                    Some(0) => {
                        self.model.state =
                            TestState::Testing(TestStep::Connecting, StepState::Waiting);
                        self.controller_message(ControllerMessage::Connect(String::from(PORT)))
                    }
                    Some(_code) => {
                        self.model.state =
                            TestState::Testing(TestStep::FlashingTest, StepState::Failed);
                    }
                    None => {
                        self.model.state =
                            TestState::Testing(TestStep::FlashingTest, StepState::Failed);
                    }
                }
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::Update(state)) => {
                self.model.digiblock_update(state);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::TestResult(step, false)) => {
                self.model.report.add_test(step.into_result(false));
                self.model.state = TestState::Testing(step, StepState::Failed);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::TestResult(step, true)) => {
                self.model.report.add_test(step.into_result(true));
                self.next_step(step);
                Command::none()
            }

            Event::ViewEvent(view::Event::UpdateDUT(val)) => {
                self.model.config.codice_dut = val;
                Command::none()
            }
            Event::ViewEvent(view::Event::UpdateOrder(val)) => {
                if str::parse::<u16>(val.as_str()).is_ok() {
                    self.model.config.ordine_forn = val;
                }
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
                self.model.state = TestState::Testing(TestStep::InvertPower, StepState::Waiting);
                self.model.report = Report::default();
                Command::perform(worker::check_power_inversion(), Event::PowerInversionCheck)
            }
            Event::ViewEvent(view::Event::Retry) => {
                match self.model.state {
                    TestState::Testing(TestStep::Connecting, _) => {
                        self.controller_message(ControllerMessage::Connect(PORT.into()))
                    }
                    TestState::Testing(TestStep::Frequency, _) => {
                        self.model.state =
                            TestState::Testing(TestStep::Frequency, StepState::Waiting);
                        self.controller_message(ControllerMessage::Test(TestStep::Frequency));
                    }
                    _ => (),
                };
                Command::none()
            }
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

        let mut subscriptions = vec![worker::worker().map(Event::ControllerEvent)];

        match self.model.state {
            TestState::Testing(TestStep::Ui, _) => {
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

    fn next_step(&mut self, step: TestStep) {
        match step {
            TestStep::Connecting => {
                self.model.state = TestState::Testing(TestStep::Ui, StepState::Waiting);
                self.model.light = RgbLight::default();
            }
            TestStep::Ui => {
                self.model.state = TestState::Testing(TestStep::Frequency, StepState::Waiting);
                self.controller_message(ControllerMessage::Test(TestStep::Frequency));
            }
            TestStep::Frequency => {
                self.model.state = TestState::Testing(TestStep::Pulses, StepState::Waiting);
                self.controller_message(ControllerMessage::Test(TestStep::Pulses));
            }
            TestStep::Pulses => {
                self.model.state = TestState::Testing(TestStep::Analog, StepState::Waiting);
                self.controller_message(ControllerMessage::Test(TestStep::Analog));
            }
            TestStep::Analog => {
                self.model.state = TestState::Testing(TestStep::Output, StepState::Waiting);
                self.controller_message(ControllerMessage::Test(TestStep::Output));
            }
            TestStep::Output => {
                self.model.state = TestState::Done;
            }
            _ => (),
        }
    }
}
