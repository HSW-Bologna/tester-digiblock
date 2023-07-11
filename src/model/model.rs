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
    Unconfigured,
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
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Configuration {
    pub codice_dut: String,
    pub operatore: String,
}

impl TestStep {
    pub fn metadata(self: &Self) -> (&'static str, &'static str, &'static str) {
        use TestStep::*;
        match self {
            InvertPower => ("A001", "Inversione di alimentazione", ""),
            FlashingTest => ("A002", "Caricamento firmware", ""),
            Connecting => ("A003", "Connessione USB", ""),
            UiRgb => ("A004", "Interfaccia utente", ""),
            Frequency => ("A005", "Lettura di una frequenza (2KHz)", "Hz"),
            Analog => ("A009", "Lettura di 420ma (4ma)", ".1mA"),
            Output => ("A0012", "Gestione uscita", ""),
            _ => ("", "", ""),
        }
    }

    pub fn limits(self: &Self) -> Option<(f64, f64)> {
        use TestStep::*;
        match self {
            InvertPower => Some((0.0, 0.0)),
            Frequency => Some((1995.0, 2005.0)),
            Analog => Some((9.8, 10.2)),
            Check3v3 => Some((3.3, 3.5)),
            Check5v => Some((4.9, 5.1)),
            Check12v => Some((12.75, 12.85)),
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
        match self.state {
            TestState::Testing(TestStep::Connecting, _) => {
                self.state = TestState::Testing(TestStep::UiLeftButton, StepState::Waiting);
                self.light = RgbLight::default();
            }
            TestState::Testing(TestStep::UiLeftButton, _) => {
                if self.digiblock_state.left_button && self.digiblock_state.right_button {
                    self.state = TestState::Testing(TestStep::UiLCD, StepState::Waiting);
                } else if self.digiblock_state.left_button {
                    self.state = TestState::Testing(TestStep::UiRightButton, StepState::Waiting);
                }
            }
            TestState::Testing(TestStep::UiRightButton, _) => {
                if self.digiblock_state.right_button {
                    self.state = TestState::Testing(TestStep::UiLCD, StepState::Waiting);
                }
            }
            _ => (),
        }
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
                    values.pop();
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
            let raw_vbat = total / values.len() as f64;
            let vbat = ((raw_vbat / 4095.0) * 3.35) * 6.0;
            Some(((vbat + 0.3) * 100.0).round() / 100.0)
        } else {
            None
        }
    }
}
