use iced;
use iced::alignment;
use iced::widget::column;
use iced::widget::*;
use iced::{Element, Length};
use iced_native::widget::scrollable::Id;

pub mod style;

use crate::model::{DigiblockResult, Model, RgbLight, TestStep};

//TODO: move away

#[derive(Clone, Debug)]
pub enum Event {
    Start,
    Retry,
    Next,
}

pub fn view<'a>(model: &'a Model) -> Element<'a, Event> {
    let title = text("Collaudo Digiblock")
        .width(Length::Fill)
        .size(32)
        .horizontal_alignment(alignment::Horizontal::Center);

    let state_view: Element<Event> = match &model.step {
        TestStep::Stopped => column![
            text("Pronto al collaudo"),
            button("Inizia").on_press(Event::Start)
        ]
        .into(),
        TestStep::InvertPower(Some(error)) => text(format!(
            "Controllo inversione alimentazione fallito: {}",
            error
        ))
        .into(),
        TestStep::InvertPower(_) => text("Controllo inversione alimentazione").into(),

        TestStep::FlashingTest(Some(error)) => {
            text(format!("Caricamento firmware fallito: {}", error)).into()
        }
        TestStep::FlashingTest(_) => text("Caricamento firmware collaudo...").into(),

        TestStep::Connecting(Some(error)) => column![
            text(format!("Connessione fallita: {}", error)),
            button("Riprova").on_press(Event::Retry),
        ]
        .into(),
        TestStep::Connecting(_) => text("Connessione...").into(),

        TestStep::Ui { light } => {
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

            column![
                buttons_text,
                light_text,
                button("Avanti").on_press(Event::Next),
            ]
            .into()
        }

        TestStep::Frequency(frequency, _error) => column![
            text(format!("Frequenza impostata: {}", frequency)),
            text(format!(
                "Frequenza letta    : {}",
                model.digiblock_state.frequency
            )),
            button("Avanti").on_press(Event::Next),
        ]
        .into(),
        TestStep::Pulses(pulses, None) => column![
            text(format!("Invio impulsi: {}", pulses)),
            text("In corso..."),
            button("Avanti").on_press(Event::Next),
        ]
        .into(),
        TestStep::Pulses(pulses, Some(Ok(received))) => column![
            text(format!("Invio impulsi: {}", pulses)),
            text(format!("Ricevuti: {}", received)),
            button("Avanti").on_press(Event::Next),
        ]
        .into(),
        TestStep::Pulses(pulses, Some(Err(()))) => column![
            text(format!("Invio impulsi: {}", pulses)),
            text("Errore!"),
            button("Avanti").on_press(Event::Next),
        ]
        .into(),

        TestStep::Analog(DigiblockResult::Waiting) => {
            column![text("Test analogico in corso")].into()
        }
        TestStep::Analog(DigiblockResult::CommunicationError) => {
            column![text("Errore di comunicazione durante il test analogico")].into()
        }
        TestStep::Analog(DigiblockResult::InvalidValue(expected, found)) => column![
            text(format!("420 ma: {}", expected)),
            text(format!("Valore non valido: {}", found)),
        ]
        .into(),
        TestStep::Analog(DigiblockResult::Ok) => column![
            text("Test analogico concluso"),
            button("Avanti").on_press(Event::Next),
        ]
        .into(),

        TestStep::Output(DigiblockResult::Waiting) => text("Test uscita in corso").into(),
        TestStep::Output(DigiblockResult::CommunicationError) => {
            text("Errore di comunicazione durante il test uscita").into()
        }
        TestStep::Output(DigiblockResult::InvalidValue((), ())) => {
            text("Test uscita fallito!").into()
        }
        TestStep::Output(DigiblockResult::Ok) => column![
            text("Test uscita concluso"),
            button("Avanti").on_press(Event::Next)
        ]
        .into(),
    };

    column![
        title,
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
