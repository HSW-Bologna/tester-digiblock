use crate::model::Model;
use chrono::{Datelike, Timelike, Utc};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;

use super::flashing::get_production_firmware_version;

const REPORTS_PATH: &str = "./reports";

pub fn save_report(model: &Model) {
    create_dir_all(REPORTS_PATH).ok();

    let now = Utc::now();
    let (_, year) = now.year_ce();
    let filename: String = format!(
        "{}/{:04}{:02}{:02}-{:02}{:02}{:02}-{}-{}-{}.yaml",
        REPORTS_PATH,
        year,
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        "",
        "digiblock2",
        if model.report.successful() {
            "PASS"
        } else {
            "FAIL"
        }
    );

    let mut file = File::create(filename).unwrap();

    let report = model
        .report
        .serializable(&model.config, get_production_firmware_version());
    //let content = serde_yaml::to_string::<SerializableReport>(&report).unwrap();

    let mut content: String = format!(
        r#"formato: {}
collaudo:
  attrezzatura: '{}'
  istanza: {}
  stazione: {}
  applicazione: '{}'
  versione: '{}'
  codice_dut: '{}'
  firmware: '{}'
  hardware: ''
  ordine_forn: {}
  fornitore: {}
  datario: ''
  pmont: '{}'
  identificativo: '{}'
  variante: '{}'
  matricola: ''
  data: '{}'
  ora: '{}'
  durata: {}
  operatore: '{}'
  esito: '{}'
  codice_di_errore: '{}'
  note: '{}'
prove:
"#,
        report.formato,
        report.collaudo.attrezzatura,
        report.collaudo.istanza,
        report.collaudo.stazione,
        report.collaudo.applicazione,
        report.collaudo.versione,
        report.collaudo.codice_dut,
        report.collaudo.firmware,
        report.collaudo.ordine_forn,
        report.collaudo.fornitore,
        report.collaudo.pmont,
        report.collaudo.identificativo,
        report.collaudo.variante,
        report.collaudo.data,
        report.collaudo.ora,
        report.collaudo.durata,
        report.collaudo.operatore,
        report.collaudo.esito,
        report.collaudo.codice_di_errore,
        report.collaudo.note,
    );

    for p in report.prove {
        content += format!(
            r#"- prova: '{}'
  descrizione: '{}'
  esito: '{}'
  durata: {}
  udm: '{}'
  valore: {}
  minimo: {}
  massimo: {}
"#,
            p.prova,
            p.descrizione.replace("'", ""),
            p.esito,
            p.durata,
            p.udm,
            yaml_nullable(p.valore),
            yaml_nullable(p.minimo),
            yaml_nullable(p.massimo),
        )
        .as_str();
    }

    file.write_all(content.replace("\n", "\r\n").as_bytes())
        .ok();
}

fn yaml_nullable<T>(value: Option<T>) -> String
where
    T: core::fmt::Display,
{
    if let Some(value) = value {
        format!("{}", value)
    } else {
        String::from("null")
    }
}
