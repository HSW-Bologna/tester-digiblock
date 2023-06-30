use iced;
use iced::alignment;
use iced::widget::column;
use iced::widget::*;
use iced::{Element, Length};
use iced_native::widget::scrollable::Id;

pub mod style;

use crate::model::{Model, RgbLight, StepState, TestState, TestStep};

//TODO: move away

#[derive(Clone, Debug)]
pub enum Event {
    Start,
    Retry,
    Done,
    SaveConfig,
    Config,
    UpdateDUT(String),
    UpdateOrder(String),
    UpdateOperator(String),
}

pub fn view<'a>(model: &'a Model) -> Element<'a, Event> {
    let title = text("Collaudo Digiblock")
        .width(Length::Fill)
        .size(32)
        .horizontal_alignment(alignment::Horizontal::Center);

    let next_button = button("Avanti").on_press(Event::Done);
    let retry_button = button("Riprova").on_press(Event::Retry);
    let config_button = button("Configura").on_press(Event::Config);

    let state_view: Element<Event> = match &model.state {
        TestState::Unconfigured => column![
            text_input("Codice DUT", model.config.codice_dut.as_str()).on_input(Event::UpdateDUT),
            text_input("Ordine", model.config.ordine_forn.as_str()).on_input(Event::UpdateOrder),
            text_input("Operatore", model.config.operatore.as_str()).on_input(Event::UpdateOperator),
            button("Salva").on_press(Event::SaveConfig),
        ]
        .into(),
        TestState::Ready => column![
            text("Pronto al collaudo"),
            row![config_button, button("Inizia").on_press(Event::Start)]
        ]
        .into(),

        TestState::Testing(TestStep::InvertPower, StepState::Waiting) => {
            text("Controllo inversione alimentazione in corso").into()
        }
        TestState::Testing(TestStep::InvertPower, StepState::Failed) => column![
            text("Controllo inversione alimentazione fallito"),
            retry_button
        ]
        .into(),

        TestState::Testing(TestStep::FlashingTest, StepState::Waiting) => {
            text("Caricamento firmware collaudo...").into()
        }
        TestState::Testing(TestStep::FlashingTest, StepState::Failed) => {
            column![text(format!("Caricamento firmware fallito")), retry_button,].into()
        }

        TestState::Testing(TestStep::Connecting, StepState::Waiting) => {
            text("Connessione...").into()
        }
        TestState::Testing(TestStep::Connecting, StepState::Failed) => {
            column![text("Connessione fallita"), retry_button, next_button].into()
        }

        TestState::Testing(TestStep::Ui, _) => {
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

            let light_text = text(match model.light {
                RgbLight::White => "bianca",
                RgbLight::Red => "rossa",
                RgbLight::Green => "Verde",
                RgbLight::Blue => "Blu",
            });

            column![buttons_text, light_text, next_button,].into()
        }

        TestState::Testing(TestStep::Frequency, StepState::Waiting) => {
            text("Test frequenza in corso").into()
        }
        TestState::Testing(TestStep::Frequency, StepState::Failed) => {
            column![text("Test frequenza fallito"), retry_button, next_button,].into()
        }

        TestState::Testing(TestStep::Pulses, StepState::Waiting) => {
            text("Invio impulsi in corso").into()
        }
        TestState::Testing(TestStep::Pulses, StepState::Failed) => {
            column![text("Test impulsi fallito"), retry_button, next_button,].into()
        }

        TestState::Testing(TestStep::Analog, StepState::Waiting) => {
            text("Test analogico in corso").into()
        }
        TestState::Testing(TestStep::Analog, StepState::Failed) => {
            column![text("Test analogico fallito"), retry_button, next_button,].into()
        }

        TestState::Testing(TestStep::Output, StepState::Waiting) => {
            text("Test uscita in corso").into()
        }
        TestState::Testing(TestStep::Output, StepState::Failed) => {
            column![text("Test uscita fallito"), retry_button, next_button,].into()
        }
        TestState::Done => column![text("Test concluso"), next_button,].into(),
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
