use serde::{Deserialize, Serialize};

use super::Report;

#[derive(Clone, Copy, Default, Debug)]
pub enum RgbLight {
    #[default]
    White,
    Red,
    Green,
    Blue,
}

#[derive(Clone, Copy, Default)]
pub enum StepState {
    #[default]
    Waiting,
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct DigiblockState {
    pub left_button: bool,
    pub right_button: bool,
    pub period_us: u16,
    pub pulses: u16,
    pub ma420: u16,
    pub short_circuit_adc: bool,
    pub short_circuit_out: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TestStep {
    InvertPower,
    FlashingTest,
    Connecting,
    UiLeftButton,
    UiRightButton,
    UiLCD,
    UiRgb,
    Check3v3,
    Check5v,
    Check12v,
    AnalogShortCircuit,
    Analog,
    Frequency,
    OutputShortCircuit,
    Output,
    FlashingProduction,
}

#[derive(Clone, Default)]
pub enum TestState {
    #[default]
    Ready,
    Testing(TestStep, StepState),
    Done,
}

#[derive(Clone, Default)]
pub struct Model {
    pub state: TestState,
    pub digiblock_state: DigiblockState,
    pub logs: Vec<String>,
    pub light: RgbLight,
    pub report: Report,
    pub config: Configuration,
    pub vbat: Option<Vec<f64>>,
    pub firmware_version: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub operatore: u8,
}

impl Default for Configuration {
    fn default() -> Self {
        Self { operatore: 1 }
    }
}

impl TestStep {
    pub fn metadata(self: &Self) -> (&'static str, &'static str, &'static str) {
        use TestStep::*;
        match self {
            InvertPower => ("A001", "Inversione della tensione", ""),
            FlashingTest => ("A002", "Caricamento firmware di collaudo", ""),
            Connecting => ("A003", "Connessione USB", ""),
            UiLeftButton => ("M001", "Verifica del funzionamento del tasto sinistro", ""),
            UiRightButton => ("M002", "Verifica del funzionamento del tasto destro", ""),
            UiLCD => ("M003", "Verifica del funzionamento dei segmenti", ""),
            UiRgb => (
                "M004",
                "Verifica del funzionamento della retroilluminazione",
                "",
            ),
            Check3v3 => ("A004", "Verifica del corretto livello della linea 3v3", "V"),
            Check5v => ("A005", "Verifica del corretto livello della linea 5v", "V"),
            Check12v => ("A006", "Verifica del corretto livello della linea 12v", "V"),
            AnalogShortCircuit => (
                "A007",
                "Verifica della rilevazione di un cortocircuito sulla linea analogica",
                "",
            ),
            Analog => (
                "A008",
                "Lettura di 420mA e verifica del valore (10mA)",
                "mA",
            ),
            Frequency => (
                "A009",
                "Lettura di una frequenza e verifica del valore",
                "Hz",
            ),
            OutputShortCircuit => (
                "A010",
                "Verifica della rilevazione di un cortocircuito sulla uscita digitale",
                "",
            ),
            Output => (
                "A0011",
                "Verifica del funzionamento della uscita digitale",
                "",
            ),
            FlashingProduction => ("A0012", "Caricamento del firmware finale", ""),
        }
    }

    pub fn limits(self: &Self) -> Option<(f64, f64)> {
        use TestStep::*;
        match self {
            InvertPower => Some((0.0, 0.0)),
            Frequency => Some((1995.0, 2005.0)),
            Analog => Some((9.75, 10.25)),
            Check3v3 => Some((3.25, 3.55)),
            Check5v => Some((4.9, 5.1)),
            Check12v => Some((11.9, 12.8)),
            _ => None,
        }
    }

    pub fn check_limits(self: &Self, value: f64) -> bool {
        if let Some((min, max)) = self.limits() {
            value >= min && value <= max
        } else {
            true // No limits, always ok
        }
    }
}

impl Model {
    pub fn logs(&self) -> String {
        let mut logs = String::new();

        for s in &self.logs {
            logs += &String::from(s.clone() + "\n");
        }

        logs
    }

    pub fn digiblock_update(&mut self, state: DigiblockState) {
        self.digiblock_state = state;
    }

    pub fn next_light(&mut self) -> RgbLight {
        self.light = match self.light {
            RgbLight::White => RgbLight::Red,
            RgbLight::Red => RgbLight::Green,
            RgbLight::Green => RgbLight::Blue,
            RgbLight::Blue => RgbLight::White,
        };
        self.light
    }

    pub fn add_vbat(&mut self, value: Option<f64>) {
        if let Some(value) = value {
            if let Some(ref mut values) = self.vbat {
                if values.len() > 10 {
                    values.remove(0);
                }
                values.push(value);
            } else {
                self.vbat = Some(vec![value]);
            }
        } else {
            self.vbat = None;
        }
    }

    pub fn get_vbat(&self) -> Option<f64> {
        if let Some(ref values) = self.vbat {
            let mut total: f64 = 0.0;
            for v in values {
                total += v;
            }
            let vbat_adc = total / values.len() as f64;

            const BASE_VALUE_ADC: f64 = 2790.0;
            const BASE_VALUE_VOLTS: f64 = 13.6;

            let vbat = (vbat_adc * BASE_VALUE_VOLTS) / BASE_VALUE_ADC;
            //let vbat = ((vbat_adc / 4095.0) * 3.35) * 6.0;
            Some((vbat * 100.0).round() / 100.0)
        } else {
            None
        }
    }

    pub fn log(self: &mut Self, msg: impl Into<String> + std::fmt::Display) {
        use chrono::prelude::*;
        let now = Utc::now();
        let time = now.format("%H:%M:%S:%3f");
        self.logs.push(format!("[{}]: {}", time, msg));
    }
}
