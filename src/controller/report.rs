use crate::model::{ SerializableReport, Model};
use chrono::{Datelike, Timelike, Utc};
use serde_yaml;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;

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

    let content =
        serde_yaml::to_string::<SerializableReport>(&model.report.serializable(&model.config))
            .unwrap();

    /*let mut content: String = format!(
            r#"formato: {}
    collaudo:
        attrezzatura: "{}"
        istanza: {}
    prove:"#,
            report.formato, report.collaudo.attrezzatura, report.collaudo.istanza,
        );

        let mut prove = Vec::from_iter(report.prove.clone());
        prove.sort_by_key(|(k, _)| k.clone());

        for el in prove {
            let (_, p) = el;
            content += format!(
                r#"
      - prova: "{}"
        esito: "{}"
    "#,
                p.prova,
                if p.esito { "Pass" } else { "Fail" }
            )
            .as_str();
        }*/

    file.write_all(content.as_bytes()).ok();
}
