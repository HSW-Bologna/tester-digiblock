use iced;
use iced::widget::column;
use iced::widget::*;
use iced::{Element, Length};
use iced_native::{Alignment, Color};

pub mod style;

use crate::model::{Model, RgbLight, StepState, TestState, TestStep};

//TODO: move away

#[derive(Clone, Debug)]
pub enum Event {
    Start,
    Retry,
    UiOk,
    UiFail,
    Done,
    UpdateOperator(u8),
    BarcodeRead(usize,String),
    BarcodeSubmit(usize),
    BarcodeReset,
}

pub fn view<'a>(model: &'a Model) -> Element<'a, Event> {
    /*let _title = text("Collaudo Digiblock")
    .width(Length::Fill)
    .size(48)
    .horizontal_alignment(alignment::Horizontal::Center);*/

    let done_button = button("Concludi").on_press(Event::Done);
    let retry_button = button("Riprova").on_press(Event::Retry);

    let barcode = &model.report.barcode;

    let state_view: Element<Event> = match &model.state {
        TestState::Ready => column![
            text("Pronto al collaudo, inserire i metadati"),
            column![
                text_input("Riferimento ordine", barcode.rif_ordine.as_str())
                    .id(text_input::Id::new("0"))
                    .on_input(|s| Event::BarcodeRead(0,s))
                    .on_submit(Event::BarcodeSubmit(0)),
                text_input("Riferimento fornitore", barcode.rif_fornitore.as_str())
                    .id(text_input::Id::new("1"))
                    .on_input(|s| Event::BarcodeRead(1,s))
                    .on_submit(Event::BarcodeSubmit(1)),
                text_input("Lotto produzione", barcode.lotto_produzione.as_str())
                    .id(text_input::Id::new("2"))
                    .on_input(|s| Event::BarcodeRead(2,s))
                    .on_submit(Event::BarcodeSubmit(2)),
                text_input("Revisione HW", barcode.rev_hw.as_str())
                    .id(text_input::Id::new("3"))
                    .on_input(|s|Event::BarcodeRead(3,s))
                    .on_submit(Event::BarcodeSubmit(3)),
                text_input("Matricola", barcode.matricola.as_str())
                    .id(text_input::Id::new("4"))
                    .on_input(|s|Event::BarcodeRead(4,s))
                    .on_submit(Event::BarcodeSubmit(4)),
                text_input("Variante", barcode.variante.as_str())
                    .id(text_input::Id::new("5"))
                    .on_input(|s|Event::BarcodeRead(5,s))
                    .on_submit(Event::BarcodeSubmit(5)),
            ]
            .align_items(Alignment::Center),
            row![
                button("Azzera").on_press(Event::BarcodeReset),
                if model.report.barcode.valid() {
                    button("Inizia").on_press(Event::Start)
                } else {
                    button("Inizia")
                }
            ]
            .spacing(128),
        ]
        .align_items(Alignment::Center)
        .spacing(32)
        .into(),
        TestState::Testing(step, state @ StepState::Waiting) => {
            test_step_description(&model, *step, *state)
        }
        TestState::Testing(step, state @ StepState::Failed) => column![
            test_step_description(&model, *step, *state),
            retry_button,
            done_button,
        ]
        .align_items(Alignment::Center)
        .spacing(32)
        .into(),
        TestState::Done => column![
            if model.report.successful() {
                text("Test concluso").style(Color::from([0.0, 0.8, 0.0]))
            } else {
                text("Test fallito").style(Color::from([0.8, 0.0, 0.0]))
            },
            done_button,
        ]
        .align_items(Alignment::Center)
        .spacing(32)
        .into(),
    };

    let power_msg = if let Some(vbat) = model.get_vbat() {
        if vbat < 13.0 || vbat > 14.2 {
            text(format!(
                "Alimentazione {:02.2}V (fuori dai valori richiesti!)",
                vbat
            ))
            .style(Color::from([0.8, 0.0, 0.0]))
        } else {
            text(format!("Alimentazione {:02.2}V", vbat))
        }
    } else {
        text(format!("Errore alimentazione")).style(Color::from([0.8, 0.0, 0.0]))
    };

    let operator_list = row![
        text("Operatore:"),
        pick_list(
            (0..99)
                .collect::<Vec<u8>>()
                .iter()
                .map(|x| x + 1)
                .collect::<Vec<u8>>(),
            Some(model.config.operatore),
            Event::UpdateOperator,
        )
    ];

    column![
        //title,
        operator_list.spacing(8.0),
        scrollable(container(state_view).width(Length::Fill).center_x())
            .height(Length::FillPortion(8)),
        power_msg,
        container(
            scrollable(text(model.logs()).width(Length::Fill)).id(scrollable::Id::new("logs"))
        )
        .padding(8)
        .width(Length::Fill)
        .height(Length::FillPortion(5))
        .style(style::bordered_container()),
    ]
    .spacing(16)
    .padding(8)
    .into()
}

fn test_step_description(model: &Model, step: TestStep, state: StepState) -> Element<Event> {
    let done_button = button("Non funzionante").on_press(Event::UiFail);
    let ok_button = button("Conferma").on_press(Event::UiOk);

    use TestStep::*;
    match (step, state) {
        (InvertPower, StepState::Waiting) => {
            column![text("Controllo inversione alimentazione in corso")]
        }
        (InvertPower, StepState::Failed) => {
            column![text("Controllo inversione alimentazione fallito")]
        }
        (FlashingTest, StepState::Waiting) => column![text("Caricamento firmware collaudo...")],
        (FlashingTest, StepState::Failed) => column![text("Caricamento firmware fallito")],
        (Connecting, StepState::Waiting) => column![text("Connessione...")],
        (Connecting, StepState::Failed) => column![text("Connessione fallita")],
        (UiLeftButton, _) => column![text("Premere il tasto sinistro"), done_button],
        (UiRightButton, _) => column![text("Premere il tasto destro"), done_button],
        (UiLCD, _) => column![
            text("Verificare che lo schermo funzioni"),
            ok_button,
            done_button
        ],
        (UiRgb, _) => {
            let light_text = text(format!(
                "Dovrebbe apparire {}",
                match model.light {
                    RgbLight::White => "bianca",
                    RgbLight::Red => "rossa",
                    RgbLight::Green => "verde",
                    RgbLight::Blue => "blu",
                }
            ));

            column![
                text("Verificare che la retroilluminazione funzioni"),
                light_text,
                ok_button,
                done_button
            ]
        }
        (Check3v3, StepState::Waiting) => column![text("Controllo alimentazione 3v3")],
        (Check3v3, StepState::Failed) => column![text("Controllo alimentazione 3v3 fallito")],
        (Check5v, StepState::Waiting) => column![text("Controllo alimentazione 5v")],
        (Check5v, StepState::Failed) => column![text("Controllo alimentazione 5v fallito")],
        (Check12v, StepState::Waiting) => column![text("Controllo alimentazione 12v")],
        (Check12v, StepState::Failed) => column![text("Controllo alimentazione 12v fallito")],
        (AnalogShortCircuit, StepState::Waiting) => {
            column![text("Test corto circuito analogico in corso")]
        }
        (AnalogShortCircuit, StepState::Failed) => {
            column![text("Test corto circuito analogico fallito")]
        }
        (Analog, StepState::Waiting) => column![text("Test analogico in corso")],
        (Analog, StepState::Failed) => column![text("Test analogico fallito")],
        (Frequency, StepState::Failed) => column![text("Test frequenza fallito")],
        (Frequency, StepState::Waiting) => column![text("Test frequenza in corso")],
        (OutputShortCircuit, StepState::Waiting) => {
            column![text("Test corto circuito uscita in corso")]
        }
        (OutputShortCircuit, StepState::Failed) => {
            column![text("Test corto circuito uscita fallito")]
        }
        (Output, StepState::Waiting) => column![text("Test uscita in corso")],
        (Output, StepState::Failed) => column![text("Test uscita fallito")],
        (FlashingProduction, StepState::Waiting) => {
            column![text("Caricamento firmware produzione...")]
        }
        (FlashingProduction, StepState::Failed) => column![text("Caricamento firmware fallito")],
    }
    .align_items(Alignment::Center)
    .spacing(32)
    .into()
}
