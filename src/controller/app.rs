use iced;
use iced::{Application, Command, Element};
use iced_native::subscription::{self, Subscription};
use iced_native::widget::scrollable::{Id, RelativeOffset};
use tokio::sync::mpsc;
use std::time::{Duration, Instant};
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;

use super::digiblock;
use crate::model::{DigiblockState, Model, RgbLight, State, TestState};
use crate::view;

#[derive(Clone, Debug)]
pub enum ControllerMessage {
    Connect(String),
    SetLight(RgbLight),
    Disconnect,
}

#[derive(Clone, Debug)]
pub enum ControllerEvent {
    Ready(mpsc::Sender<ControllerMessage>),
    Log(String),
    Connected,
    Disconnected,
    Update(DigiblockState),
}

#[derive(Clone, Debug)]
pub enum Event {
    UpdatePorts(Instant),
    UpdateLight(Instant),
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
                model: Model {
                    ports: get_ports(),
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
            Event::ControllerEvent(ControllerEvent::Connected) => {
                self.model.state = State::Connected(TestState::Unresponsive);
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::Disconnected) => {
                self.model.state = State::Disconnected;
                Command::none()
            }
            Event::ControllerEvent(ControllerEvent::Log(msg)) => {
                self.model.logs.push(msg);
                iced::widget::scrollable::snap_to(Id::new("logs"), RelativeOffset::END)
            }
            Event::ControllerEvent(ControllerEvent::Update(state)) => {
                self.model.digiblock_update(state);
                Command::none()
            }
            Event::UpdatePorts(_) => {
                self.model.update_ports(get_ports());
                Command::none()
            }
            Event::UpdateLight(_) => {
                let light = self.model.next_light();
                self.controller_message(ControllerMessage::SetLight(light));
                Command::none()
            }
            Event::ViewEvent(view::Event::Connect(port)) => {
                self.controller_message(ControllerMessage::Connect(port));
                Command::none()
            }
            Event::ViewEvent(view::Event::Disconnect) => {
                self.controller_message(ControllerMessage::Disconnect);
                Command::none()
            }
            Event::ViewEvent(view::Event::SelectedPort(port)) => {
                self.model.selected_port = Some(port.clone());
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

        match self.model.state {
            State::Connected(TestState::Ui { light }) => {
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
        msg: &str,
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
        Connected(tokio_modbus::client::Context),
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
                                    let builder = tokio_serial::new(port, 115200);
                                    if let Ok(port) = SerialStream::open(&builder) {
                                        if let Ok(ctx) = tokio_modbus::client::rtu::connect_slave(
                                            port,
                                            Slave(0x01),
                                        )
                                        .await
                                        {
                                            log(&mut output, "Connection successful").await;
                                            output.send(ControllerEvent::Connected).await.ok();
                                            state = State::Connected(ctx);
                                        } else {
                                            log(&mut output, "Connection failed").await;
                                        }
                                    } else {
                                        log(&mut output, "Connection failed").await;
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
                            match msg {
                                ControllerMessage::Disconnect => {
                                    ctx.disconnect().await.ok();
                                    output.send(ControllerEvent::Disconnected).await.ok();
                                    state = State::Disconnected;
                                }
                                ControllerMessage::SetLight(light) => {
                                    digiblock::set_light(ctx, light).await.ok();
                                }
                                _ => (),
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
                                    output.send(ControllerEvent::Disconnected).await.ok();
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

fn get_ports() -> Vec<String> {
    serialport::available_ports()
        .unwrap_or(Vec::new())
        .iter()
        .map(|p| p.port_name.clone())
        .collect()
}
