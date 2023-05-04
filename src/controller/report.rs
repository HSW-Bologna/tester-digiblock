use crate::model::Report;
use chrono::{Datelike, Timelike, Utc};
use std::fs::File;
use std::io::prelude::*;

pub fn save_report(report: &Report) {
    let now = Utc::now();
    let (_, year) = now.year_ce();
    let filename: String = format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}-{}-{}-{}.yaml",
        year,
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        report
            .serial_number
            .map(|v| format!("{}", v))
            .unwrap_or(String::from("AXXXXXX")),
        report.code,
        if report.success { "PASS" } else { "FAIL" }
    );

    let mut file = File::create(filename).unwrap();
    file.write_all(
        serde_yaml::to_string(&report.report_data)
            .unwrap()
            .as_bytes(),
    )
    .ok();
}
