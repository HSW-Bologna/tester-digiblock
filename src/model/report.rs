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
    pub matricola: String,
    pub data: String,
    pub ora: String,
    pub durata: f64,
    pub operatore: String,
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

    pub fn serializable(self: &Self, config: &Configuration) -> SerializableReport {
        let mut prove: Vec<SerializableTestStepResult> = Vec::new();

        const TESTS: [TestStep; 10] = [
            TestStep::InvertPower,
            TestStep::FlashingTest,
            TestStep::Connecting,
            TestStep::UiLeftButton,
            TestStep::UiRightButton,
            TestStep::UiLCD,
            TestStep::UiRgb,
            TestStep::Frequency,
            TestStep::Analog,
            TestStep::Output,
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
            if p.esito != "Pass" {
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
            formato: 1,
            collaudo: TestStation {
                attrezzatura,
                istanza,
                stazione: 1,
                applicazione: NAME.into(),
                versione: VERSION.into(),
                codice_dut: config.codice_dut.clone(),
                firmware: "TODO".into(),
                matricola: "TODO".into(),
                data: format!(
                    "{}-{}-{}",
                    self.start.year(),
                    self.start.month(),
                    self.start.day()
                ),
                ora: format!(
                    "{}-{}-{}",
                    self.start.hour(),
                    self.start.minute(),
                    self.start.second()
                ),
                durata: end.sub(self.start).num_seconds() as f64,
                operatore: config.operatore.clone(),
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
