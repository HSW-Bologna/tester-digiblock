use iced;
use iced::widget::text_input;
use iced::{Application, Command, Element};
use iced_native::widget::scrollable::{Id, RelativeOffset};
use std::fs;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::{flashing, worker};
use super::{reles, save_report};
use crate::controller::adc;
use crate::model::{
    Barcode, Configuration, DigiblockState, Model, Report, RgbLight, StepState, TestState,
    TestStep, TestStepResult,
};
use crate::view;

const PORT: &str = "/dev/ttyACM0";
const CONFIG: &str = "./config.yaml";
const BASE_VARIANT: &str = "1";

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
                self.sender = Some(sender);
                text_input::focus(text_input::Id::new("0"))
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
                let vbat = worker::read_vbat().ok();
                self.model.add_vbat(vbat);
                if vbat.is_some() {
                    //println!("VBat {:?}", vbat.unwrap());
                }
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::Update(state)) => {
                self.model.digiblock_update(state);

                match self.model.state {
                    TestState::Testing(TestStep::Connecting, _) => {
                        self.model.state =
                            TestState::Testing(TestStep::UiLeftButton, StepState::Waiting);
                        self.model.light = RgbLight::default();
                    }
                    TestState::Testing(TestStep::UiLeftButton, _) => {
                        if self.model.digiblock_state.left_button
                            && self.model.digiblock_state.right_button
                        {
                            self.model.log("Tasto sinistro rilevato");
                            self.model.log("Tasto destro rilevato");
                            self.add_test(TestStep::UiLeftButton, true, None);
                            self.add_test(TestStep::UiRightButton, true, None);
                            self.start_ts = Instant::now();
                            self.model.state =
                                TestState::Testing(TestStep::UiLCD, StepState::Waiting);
                        } else if self.model.digiblock_state.left_button {
                            self.model.log("Tasto sinistro rilevato");
                            self.add_test(TestStep::UiLeftButton, true, None);
                            self.start_ts = Instant::now();
                            self.model.state =
                                TestState::Testing(TestStep::UiRightButton, StepState::Waiting);
                        }
                    }
                    TestState::Testing(TestStep::UiRightButton, _) => {
                        if self.model.digiblock_state.right_button {
                            self.model.log("Tasto destro rilevato");
                            self.add_test(TestStep::UiRightButton, true, None);
                            self.start_ts = Instant::now();
                            self.model.state =
                                TestState::Testing(TestStep::UiLCD, StepState::Waiting);
                        }
                    }
                    _ => (),
                }

                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::TestResult(step, value, success)) => {
                use TestStep::*;
                match step {
                    InvertPower => {
                        self.model.log(if success {
                            "Alimentazione invertita con successo"
                        } else {
                            "Consumo eccessivo su alimentazione invertita"
                        });
                    }
                    FlashingTest => {
                        self.model.log(if success {
                            "Firmware di collaudo caricato"
                        } else {
                            "Caricamento firmware di collaudo fallito"
                        });
                    }
                    Connecting => {
                        self.model.log(if success {
                            "Connessione effettuata"
                        } else {
                            "Connessione fallita"
                        });
                    }
                    AnalogShortCircuit => {
                        self.model.log(if success {
                            "Corto circuito analogico rilevato"
                        } else {
                            "Corto circuito analogico non rilevato"
                        });
                    }
                    Analog => {
                        self.model.log(format!(
                            "Valore analogico: 10 mA - {} mA",
                            value.map(|x| x.to_string()).unwrap_or("---".into())
                        ));
                    }
                    Frequency => {
                        self.model.log(format!(
                            "Frequenza: 2000 Hz - {} Hz",
                            value.map(|x| x.to_string()).unwrap_or("---".into())
                        ));
                    }
                    OutputShortCircuit => {
                        self.model.log(if success {
                            "Corto circuito digitale rilevato"
                        } else {
                            "Corto circuito digitale non rilevato"
                        });
                    }
                    Output => {
                        self.model.log(if success {
                            "Collaudo uscita riuscito"
                        } else {
                            "Collaudo uscita fallito"
                        });
                    }
                    FlashingProduction => {
                        self.model.log(if success {
                            "Firmware di produzione caricato"
                        } else {
                            "Caricamento firmware di produzione fallito"
                        });
                    }

                    _ => (),
                }

                self.add_test(step, success, value);

                let scroll_cmd =
                    iced::widget::scrollable::snap_to(Id::new("logs"), RelativeOffset::END);
                if success {
                    iced::Command::batch(vec![self.next_step(step), scroll_cmd])
                } else {
                    self.model.state = TestState::Testing(step, StepState::Failed);
                    scroll_cmd
                }
            }

            Event::ViewEvent(view::Event::UpdateOperator(val)) => {
                if val > 0 && val < 100 {
                    self.model.config.operatore = val;
                    fs::write(CONFIG, serde_yaml::to_string(&self.model.config).unwrap()).unwrap();
                }
                text_input::focus(text_input::Id::new("0"))
            }
            Event::ViewEvent(view::Event::BarcodeRead(index, val)) => {
                self.model
                    .report
                    .barcode
                    .change_field_num(index, val.clone());
                Command::none()
            }
            Event::ViewEvent(view::Event::BarcodeSubmit(index)) => {
                if index >= 5 {
                    //self.start_procedure()
                    Command::none()
                } else {
                    text_input::focus(text_input::Id::new((index+1).to_string()))
                }
            }
            Event::ViewEvent(view::Event::BarcodeReset) => {
                self.model.report.barcode = Barcode::default();
                text_input::focus(text_input::Id::new("0"))
            }

            Event::ViewEvent(view::Event::Start) => self.start_procedure(),
            Event::ViewEvent(view::Event::Retry) => {
                use TestStep::*;
                match self.model.state {
                    TestState::Testing(FlashingTest, _) => self.flash_test_firmware(),
                    TestState::Testing(Connecting, _) => {
                        self.controller_message(ControllerMessage::Connect(PORT.into()));
                        Command::none()
                    }
                    TestState::Testing(InvertPower, _) => {
                        self.model.state =
                            TestState::Testing(TestStep::InvertPower, StepState::Waiting);
                        Self::perform_power_inversion()
                    }
                    TestState::Testing(Check3v3 | Check5v | Check12v, _) => {
                        if self.test_power().is_ok() {
                            self.start_test(AnalogShortCircuit)
                        } else {
                            iced::widget::scrollable::snap_to(Id::new("logs"), RelativeOffset::END)
                        }
                    }
                    TestState::Testing(FlashingProduction, _) => self.flash_production_firmware(),
                    TestState::Testing(step, _) => {
                        self.model.state = TestState::Testing(step, StepState::Waiting);
                        self.controller_message(ControllerMessage::Test(step));
                        Command::none()
                    }
                    _ => Command::none(),
                }
            }
            Event::ViewEvent(view::Event::UiFail) => {
                match self.model.state {
                    TestState::Testing(step, _) => {
                        self.add_test(step, false, None);
                        self.model.state = TestState::Done;
                    }
                    _ => (),
                }
                Command::none()
            }
            Event::ViewEvent(view::Event::UiOk) => match self.model.state {
                TestState::Testing(TestStep::UiLCD, _) => {
                    self.model.log("LCD funzionante");
                    self.add_test(TestStep::UiLCD, true, None);
                    self.next_step(TestStep::UiLCD)
                }
                _ => {
                    self.model.log("RGB funzionante");
                    self.add_test(TestStep::UiRgb, true, None);
                    self.next_step(TestStep::UiRgb)
                }
            },
            Event::ViewEvent(view::Event::Done) => {
                save_report(&self.model);

                self.model.logs = vec![];
                self.model.state = TestState::Ready;
                self.controller_message(ControllerMessage::Disconnect);
                reles::all_off();
                self.model.report = Report::default();

                text_input::focus(text_input::Id::new("0"))
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
            .log(format!("Tensione su linea 3v3: {}V", power3v3));

        if TestStep::Check3v3.check_limits(power3v3) {
            self.add_test(TestStep::Check3v3, true, Some(power3v3 as f64));
        } else {
            self.add_test(TestStep::Check3v3, false, Some(power3v3 as f64));
            self.model.state = TestState::Testing(TestStep::Check3v3, StepState::Failed);
            return Err(());
        }

        if self.model.report.barcode.variante == BASE_VARIANT {
            self.model.log(format!(
                "Modello base, salto il controllo della tensione 5v"
            ));
        } else {
            let power5v = adc::read_adc(adc::Channel::Volt5).map_err(|_| ())?;
            println!("{}", power5v);
            let power5v = ((power5v as f64 / 4095.0) * 3.35) * 2.0;
            let power5v = (power5v * 100.0).round() / 100.0;

            self.model
                .log(format!("Tensione su linea 5v: {}V", power5v));

            if TestStep::Check5v.check_limits(power5v) {
                self.add_test(TestStep::Check5v, true, Some(power5v as f64));
            } else {
                self.add_test(TestStep::Check5v, false, Some(power5v as f64));
                self.model.state = TestState::Testing(TestStep::Check5v, StepState::Failed);
                return Err(());
            }
        }

        let power12v = adc::read_adc(adc::Channel::Supply).map_err(|_| ())?;
        let power12v = ((power12v as f64 / 4095.0) * 3.35) * (13.43 / 1.43);
        let power12v = (power12v * 100.0).round() / 100.0;

        self.model
            .log(format!("Tensione su linea 12v: {}V", power12v));

        if TestStep::Check12v.check_limits(power12v) {
            self.add_test(TestStep::Check12v, true, Some(power12v as f64));
            Ok(())
        } else {
            self.add_test(TestStep::Check12v, false, Some(power12v as f64));
            self.model.state = TestState::Testing(TestStep::Check12v, StepState::Failed);
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
        Command::perform(
            async {
                let res = flashing::load_test_firmware().await;
                if res == Some(0) {
                    worker::reset().await;
                }
                res
            },
            |res| {
                ControllerEvent::TestResult(
                    TestStep::FlashingTest,
                    None,
                    if let Some(code) = res {
                        code == 0
                    } else {
                        false
                    },
                )
            },
        )
        .map(Event::ControllerEvent)
    }

    fn flash_production_firmware(self: &mut Self) -> Command<Event> {
        self.controller_message(ControllerMessage::Disconnect);

        self.model.state = TestState::Testing(TestStep::FlashingProduction, StepState::Waiting);

        Command::perform(
            async {
                tokio::time::sleep(Duration::from_millis(500)).await;
                // Shitty busy loop to make sure we wait for disconnection
                flashing::load_production_firmware().await
            },
            |res| {
                ControllerEvent::TestResult(
                    TestStep::FlashingProduction,
                    None,
                    if let Some(code) = res {
                        code == 0
                    } else {
                        false
                    },
                )
            },
        )
        .map(Event::ControllerEvent)
    }

    fn perform_power_inversion() -> Command<Event> {
        Command::perform(worker::check_power_inversion(), |r| {
            let (_value, success) = if let Ok(value) = r {
                (Some(value), value < 2050.0)
            } else {
                (None, false)
            };

            Event::ControllerEvent(ControllerEvent::TestResult(
                TestStep::InvertPower,
                None,
                success,
            ))
        })
    }

    fn start_procedure(self: &mut Self) -> Command<Event> {
        self.start_ts = Instant::now();
        self.model.state = TestState::Testing(TestStep::InvertPower, StepState::Waiting);
        Self::perform_power_inversion()
    }
}
