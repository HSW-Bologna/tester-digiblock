use iced;
use iced::alignment;
use iced::widget::column;
use iced::widget::*;
use iced::{Element, Length};
use iced_native::widget::scrollable::Id;

pub mod style;

use crate::model::{Model, RgbLight, State, TestState};

//TODO: move away

#[derive(Clone, Debug)]
pub enum Event {
    Connect(String),
    Disconnect,
    SelectedPort(String),
}

pub fn view<'a>(model: &'a Model) -> Element<'a, Event> {
    let title = text("Collaudo Digiblock")
        .width(Length::Fill)
        .size(32)
        .horizontal_alignment(alignment::Horizontal::Center);

    let ports_picker = pick_list(
        model.ports.clone(),
        model.selected_port.clone(),
        Event::SelectedPort,
    )
    .placeholder("Selezionare porta");

    let state_view: Element<Event> = match model.state {
        State::Disconnected => text("Non connesso").into(),
        State::Connected(TestState::Unresponsive) => text("La scheda non risponde").into(),
        State::Connected(TestState::Ui { light }) => {
            let buttons_text = text(
                if model.digiblock_state.left_button && model.digiblock_state.right_button {
                    "Pulsanti funzionanti"
                } else if model.digiblock_state.right_button {
                    "Premere il pulsante destro"
                } else if model.digiblock_state.left_button {
                    "Premere il pulsante destro"
                } else {
                    "Premere i pulsanti"
                },
            );

            let light_text = text(match light {
                RgbLight::White => "bianca",
                RgbLight::Red => "rossa",
                RgbLight::Green => "Verde",
                RgbLight::Blue => "Blu",
            });

            column![buttons_text, light_text].into()
        }
    };

    let connect_button = if model.connected() {
        button(text("Disconnetti")).on_press(Event::Disconnect)
    } else if let Some(port) = &model.selected_port {
        button(text("Connetti")).on_press(Event::Connect(port.clone()))
    } else {
        button(text("Connetti"))
    };

    column![
        title,
        row![ports_picker, connect_button,],
        scrollable(
            container(state_view)
                .width(Length::Fill)
                .padding(16)
                .center_x()
        )
        .height(Length::Fill),
        container(scrollable(text(model.logs()).width(Length::Fill)).id(Id::new("logs")))
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fixed(128.0))
            .style(style::bordered_container()),
    ]
    .spacing(16)
    .into()
}
