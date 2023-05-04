use serde::Serialize;
use std::fmt;

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

pub struct Report {
    pub serial_number: Option<SerialNumber>,
    pub code: String,
    pub success: bool,
    pub report_data: ReportData,
}

#[derive(Serialize)]
pub struct ReportData {
    formato: u8,
}

impl Default for ReportData {
    fn default() -> Self {
        ReportData { formato: 1 }
    }
}

impl Default for Report {
    fn default() -> Self {
        Report {
            serial_number: None,
            code: String::new(),
            success: false,
            report_data: ReportData::default(),
        }
    }
}
