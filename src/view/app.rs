use iced;
use iced::widget::column;
use iced::widget::{button, text};
use iced::{Application, Command, Element};

use crate::model::Model;

#[derive(Clone, Debug)]
pub enum Message {
    Increase,
}

pub struct App {
    model: Model,
}

impl Application for App {
    type Message = Message;
    type Theme = iced::theme::Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (
            App {
                model: Model::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Digiblock Test")
    }

    fn view(&self) -> Element<Message> {
        // We use a column: a simple vertical layout
        column![
            // The increment button. We tell it to produce an
            // `IncrementPressed` message when pressed
            button("+").on_press(Message::Increase),
            // We show the value of the counter here
            text(self.model.count).size(50),
            // The decrement button. We tell it to produce a
            // `DecrementPressed` message when pressed
            button("-").on_press(Message::Increase),
        ]
        .into()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Increase => {
                self.model.count += 1;
                Command::none()
            }
        }
    }

    fn theme(&self) -> iced::theme::Theme {
        iced::theme::Theme::Dark
    }
}
