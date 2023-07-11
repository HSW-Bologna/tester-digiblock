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
    UiOk,
    Done,
    SaveConfig,
    Config,
    UpdateDUT(String),
    UpdateOperator(String),
}

pub fn view<'a>(model: &'a Model) -> Element<'a, Event> {
    let title = text("Collaudo Digiblock")
        .width(Length::Fill)
        .size(48)
        .horizontal_alignment(alignment::Horizontal::Center);

    let done_button = button("Concludi").on_press(Event::Done);
    let retry_button = button("Riprova").on_press(Event::Retry);
    let config_button = button("Configura").on_press(Event::Config);

    let state_view: Element<Event> = match &model.state {
        TestState::Unconfigured => column![
            text_input("Codice DUT", model.config.codice_dut.as_str()).on_input(Event::UpdateDUT),
            text_input("Operatore", model.config.operatore.as_str())
                .on_input(Event::UpdateOperator),
            button("Salva").on_press(Event::SaveConfig),
        ]
        .into(),
        TestState::Ready => column![
            text("Pronto al collaudo"),
            row![config_button, button("Inizia").on_press(Event::Start)]
        ]
        .into(),

        TestState::Testing(step, state @ StepState::Waiting) => {
            test_step_description(&model, *step, *state)
        }
        TestState::Testing(step, state @ StepState::Failed) => column![
            test_step_description(&model, *step, *state),
            row![retry_button, done_button]
        ]
        .into(),
        TestState::Done => column![text("Test concluso"), done_button,].into(),
    };

    let power_msg = if let Some(vbat) = model.get_vbat() {
        format!("Alimentazione {:02}", vbat)
    } else {
        format!("Errore alimentazione")
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
        text(power_msg),
        container(scrollable(text(model.logs()).width(Length::Fill)).id(Id::new("logs")))
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fixed(256.0))
            .style(style::bordered_container()),
    ]
    .spacing(16)
    .into()
}

fn test_step_description(model: &Model, step: TestStep, state: StepState) -> Element<Event> {
    let done_button = button("Non funzionante").on_press(Event::Done);
    let ok_button = button("Conferma").on_press(Event::UiOk);

    use TestStep::*;
    match (step, state) {
        (InvertPower, StepState::Waiting) => {
            text("Controllo inversione alimentazione in corso").into()
        }
        (InvertPower, StepState::Failed) => {
            text("Controllo inversione alimentazione fallito").into()
        }
        (FlashingTest, StepState::Waiting) => text("Caricamento firmware collaudo...").into(),
        (FlashingTest, StepState::Failed) => text("Caricamento firmware fallito").into(),
        (Connecting, StepState::Waiting) => text("Connessione...").into(),
        (Connecting, StepState::Failed) => text("Connessione fallita").into(),
        (UiLeftButton, _) => column![text("Premere il tasto sinistro"), done_button].into(),
        (UiRightButton, _) => column![text("Premere il tasto destro"), done_button].into(),
        (UiLCD, _) => column![
            text("Verificare che lo schermo funzioni"),
            ok_button,
            done_button
        ]
        .into(),
        (UiRgb, _) => {
            let light_text = text(match model.light {
                RgbLight::White => "bianca",
                RgbLight::Red => "rossa",
                RgbLight::Green => "Verde",
                RgbLight::Blue => "Blu",
            });

            column![
                text("Verificare che la retroilluminazione funzioni"),
                light_text,
                ok_button,
                done_button
            ]
            .into()
        }
        (Check3v3, StepState::Waiting) => text("Controllo alimentazione 3v3").into(),
        (Check3v3, StepState::Failed) => text("Controllo alimentazione 3v3 fallito").into(),
        (Check5v, StepState::Waiting) => text("Controllo alimentazione 5v").into(),
        (Check5v, StepState::Failed) => text("Controllo alimentazione 5v fallito").into(),
        (Check12v, StepState::Waiting) => text("Controllo alimentazione 12v").into(),
        (Check12v, StepState::Failed) => text("Controllo alimentazione 12v fallito").into(),
        (AnalogShortCircuit, StepState::Waiting) => {
            text("Test corto circuito analogico in corso").into()
        }
        (AnalogShortCircuit, StepState::Failed) => {
            text("Test corto circuito analogico fallito").into()
        }
        (Analog, StepState::Waiting) => text("Test analogico in corso").into(),
        (Analog, StepState::Failed) => text("Test analogico fallito").into(),
        (Frequency, StepState::Failed) => text("Test frequenza fallito").into(),
        (Frequency, StepState::Waiting) => text("Test frequenza in corso").into(),
        (OutputShortCircuit, StepState::Waiting) => {
            text("Test corto circuito uscita in corso").into()
        }
        (OutputShortCircuit, StepState::Failed) => {
            text("Test corto circuito uscita fallito").into()
        }
        (Output, StepState::Waiting) => text("Test uscita in corso").into(),
        (Output, StepState::Failed) => text("Test uscita fallito").into(),
        (FlashingProduction, StepState::Waiting) => {
            text("Caricamento firmware produzione...").into()
        }
        (FlashingProduction, StepState::Failed) => text("Caricamento firmware fallito").into(),
    }
}
