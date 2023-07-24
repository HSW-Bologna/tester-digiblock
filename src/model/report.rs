use std::fs;
use std::ops::Sub;
use std::{collections::HashMap, fmt, time::Duration};

use chrono::{DateTime, Datelike, Local, Timelike};
use serde::Serialize;

use super::{Configuration, TestStep};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Clone, Copy)]
pub struct SerialNumber {
    character: char,
    numbers: [u8; 6],
}

impl fmt::Display for SerialNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}{}{}{}",
            self.character,
            self.numbers[0],
            self.numbers[1],
            self.numbers[2],
            self.numbers[3],
            self.numbers[4],
            self.numbers[5]
        )
    }
}

#[derive(Clone)]
pub struct Report {
    pub start: DateTime<Local>,
    pub tests: HashMap<TestStep, TestStepResult>,
    pub barcode: Barcode,
}

#[derive(Clone, Serialize, Default)]
pub struct Barcode {
    pub rif_ordine: String,
    pub rif_fornitore: String,
    pub lotto_produzione: String,
    pub rev_hw: String,
    pub matricola: String,
    pub variante: String,
}

impl Barcode {
    pub fn change_field_num(self: &mut Self, index: usize, field: String) {
        match index {
            0 => self.rif_ordine = field,
            1 => self.rif_fornitore = field,
            2 => self.lotto_produzione = field,
            3 => self.rev_hw = field,
            4 => self.matricola = field,
            5 => self.variante = field,
            _ => (),
        }
    }

    pub fn valid(self: &Self) -> bool {
        !(self.rif_ordine.is_empty()
            || self.rif_fornitore.is_empty()
            || self.lotto_produzione.is_empty()
            || self.rev_hw.is_empty()
            || self.matricola.is_empty()
            || self.variante.is_empty())
    }
}

#[derive(Clone)]
pub struct TestStepResult {
    pub step: TestStep,
    pub success: bool,
    pub value: Option<f64>,
    pub duration: Duration,
}

#[derive(Serialize)]
pub struct SerializableReport {
    pub formato: u8,
    pub collaudo: TestStation,
    pub prove: Vec<SerializableTestStepResult>,
}

#[derive(Serialize)]
pub struct TestStation {
    pub attrezzatura: String,
    pub istanza: u16,
    pub stazione: u16,
    pub applicazione: String,
    pub versione: String,
    pub codice_dut: String,
    pub firmware: String,
    pub pmont: String,
    pub ordine_forn: u64,
    pub fornitore: u64,
    pub identificativo: String,
    pub data: String,
    pub ora: String,
    pub durata: f64,
    pub operatore: String,
    pub variante: String,
    pub esito: String,
    pub codice_di_errore: String,
    pub note: String,
}

#[derive(Serialize)]
pub struct SerializableTestStepResult {
    pub prova: String,
    pub descrizione: String,
    pub esito: String,
    pub durata: f32,
    pub udm: String,
    pub valore: Option<f64>,
    pub minimo: Option<f64>,
    pub massimo: Option<f64>,
}

impl Into<SerializableTestStepResult> for TestStepResult {
    fn into(self) -> SerializableTestStepResult {
        let (name, description, udm) = self.step.metadata();
        let (minimo, massimo) = if let Some((minimo, massimo)) = self.step.limits() {
            (Some(minimo), Some(massimo))
        } else {
            (None, None)
        };

        SerializableTestStepResult {
            prova: name.into(),
            descrizione: description.into(),
            esito: (if self.success { "Pass" } else { "Fail" }).into(),
            durata: ((self.duration.as_secs_f32() * 10.0).round()) / 10.0,
            udm: udm.into(),
            valore: self.value,
            minimo,
            massimo,
        }
    }
}

impl SerializableTestStepResult {
    pub fn unexecuted(step: TestStep) -> Self {
        let (name, description, udm) = step.metadata();
        let (minimo, massimo) = if let Some((minimo, massimo)) = step.limits() {
            (Some(minimo), Some(massimo))
        } else {
            (None, None)
        };

        SerializableTestStepResult {
            prova: name.into(),
            descrizione: description.into(),
            esito: "Unexecuted".into(),
            durata: 0.0,
            udm: udm.into(),
            valore: None,
            minimo,
            massimo,
        }
    }
}

impl Default for Report {
    fn default() -> Self {
        Self {
            start: chrono::offset::Local::now(),
            tests: HashMap::new(),
            barcode: Barcode::default(),
        }
    }
}

impl Report {
    pub fn successful(self: &Self) -> bool {
        self.tests
            .clone()
            .into_iter()
            .fold(true, |acc, (_key, val)| acc && val.success)
    }

    pub fn add_test(self: &mut Self, test: TestStepResult) {
        self.tests.insert(test.step, test);
    }

    pub fn serializable(
        self: &Self,
        config: &Configuration,
        version: String,
    ) -> SerializableReport {
        let mut prove: Vec<SerializableTestStepResult> = Vec::new();

        const TESTS: [TestStep; 16] = [
            TestStep::InvertPower,
            TestStep::FlashingTest,
            TestStep::Connecting,
            TestStep::UiLeftButton,
            TestStep::UiRightButton,
            TestStep::UiLCD,
            TestStep::UiRgb,
            TestStep::Check3v3,
            TestStep::Check5v,
            TestStep::Check12v,
            TestStep::AnalogShortCircuit,
            TestStep::Analog,
            TestStep::Frequency,
            TestStep::OutputShortCircuit,
            TestStep::Output,
            TestStep::FlashingProduction,
        ];

        for step in TESTS {
            if let Some(result) = self.tests.get(&step) {
                prove.push(result.clone().into());
            } else {
                prove.push(SerializableTestStepResult::unexecuted(step));
            }
        }

        let end = chrono::offset::Local::now();

        let mut codice_di_errore = String::new();
        for p in &prove {
            if p.esito == "Fail" {
                codice_di_errore = p.prova.clone();
                break;
            }
        }

        let attrezzatura = "BC033".into();
        let istanza = fs::read_to_string("~/istanza_attrezzatura.txt")
            .unwrap_or("".into())
            .parse()
            .unwrap_or(1);

        SerializableReport {
            formato: 2,
            collaudo: TestStation {
                attrezzatura,
                istanza,
                stazione: 1,
                applicazione: NAME.into(),
                versione: VERSION.into(),
                codice_dut: "digiblock2".into(),
                firmware: version,
                pmont: self.barcode.rev_hw.clone(),
                ordine_forn: self.barcode.rif_ordine.parse().unwrap_or(0),
                fornitore: self.barcode.rif_fornitore.parse().unwrap_or(0),
                identificativo: self.barcode.matricola.clone(),
                variante: self.barcode.variante.clone(),
                data: format!(
                    "{:02}-{:02}-{:02}",
                    self.start.year(),
                    self.start.month(),
                    self.start.day()
                ),
                ora: format!(
                    "{:02}:{:02}:{:02}",
                    self.start.hour(),
                    self.start.minute(),
                    self.start.second()
                ),
                durata: end.sub(self.start).num_seconds() as f64,
                operatore: format!("MB_OP{:02}", config.operatore),
                esito: (if self.successful() { "Pass" } else { "Fail" }).into(),
                codice_di_errore,
                note: "".into(),
            },
            prove,
        }
    }
}

impl TestStepResult {
    pub fn new(step: TestStep, success: bool, value: Option<f64>, duration: Duration) -> Self {
        Self {
            step,
            success,
            value,
            duration,
        }
    }
}
