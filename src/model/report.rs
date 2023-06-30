use std::fs;
use std::ops::Sub;
use std::{collections::HashMap, fmt};

use chrono::{DateTime, Datelike, Local, Timelike};
use serde::Serialize;

use super::Configuration;

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
    pub tests: HashMap<String, TestStepResult>,
}

#[derive(Clone, Default)]
pub struct TestStepResult {
    pub name: String,
    pub success: bool,
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
    pub hardware: String,
    pub ordine_forn: u16,
    pub datario: String,
    pub pmont: String,
    pub identificativo: String,
    pub variante: String,
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
    pub esito: String,
}

impl Into<SerializableTestStepResult> for TestStepResult {
    fn into(self) -> SerializableTestStepResult {
        SerializableTestStepResult {
            prova: self.name,
            esito: (if self.success { "Pass" } else { "Fail" }).into(),
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
        self.tests.insert(test.name.clone(), test);
    }

    pub fn serializable(self: &Self, config: &Configuration) -> SerializableReport {
        let mut prove: Vec<SerializableTestStepResult> =
            Vec::from_iter(self.tests.clone().into_iter().map(|(_, v)| v.into()));
        prove.sort_by_key(|v| v.prova.clone());

        let end = chrono::offset::Local::now();

        let mut codice_di_errore = String::new();
        for p in &prove {
            if p.esito != "Pass" {
                codice_di_errore = p.prova.clone();
                break;
            }
        }

        let attrezzatura = fs::read_to_string("~/codice_attrezzatura.txt").unwrap_or("".into());
        let istanza = fs::read_to_string("~/istanza_attrezzatura.txt")
            .unwrap_or("".into())
            .parse()
            .unwrap_or(1);

        let now = chrono::offset::Local::now();
        let datario = format!(
            "{:02}{:02}",
            now.iso_week().week(),
            now.iso_week().year() % 100
        );

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
                hardware: "1".into(),
                ordine_forn: str::parse(config.ordine_forn.as_str()).unwrap_or(0),
                datario,
                pmont: "".into(),
                identificativo: "".into(),
                variante: "".into(),
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
    pub fn description(name: &str, result: bool) -> Self {
        Self {
            name: name.into(),
            success: result,
            ..Self::default()
        }
    }
}
