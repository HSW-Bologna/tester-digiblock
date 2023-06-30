use serde::{Serialize, Deserialize};

use super::{Report, TestStepResult};

#[derive(Clone, Copy, Default, Debug)]
pub enum RgbLight {
    #[default]
    White,
    Red,
    Green,
    Blue,
}

#[derive(Clone, Default)]
pub enum StepState {
    #[default]
    Waiting,
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct DigiblockState {
    pub left_button: bool,
    pub right_button: bool,
    pub frequency: u16,
    pub pulses: u16,
    pub ma420: u16,
}

#[derive(Clone, Debug)]
pub enum TestStep {
    InvertPower,
    FlashingTest,
    Connecting,
    Ui,
    Frequency,
    Pulses,
    Analog,
    Output,
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
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Configuration {
    pub codice_dut: String,
    pub ordine_forn: String,
    pub operatore: String,
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
                self.state = TestState::Testing(TestStep::Ui, StepState::Waiting);
                self.light = RgbLight::default();
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
}

impl TestStep {
    pub fn into_result(self: &Self, result: bool) -> TestStepResult {
        use TestStep::*;
        match self {
            InvertPower => TestStepResult::description("A001", result),
            FlashingTest => TestStepResult::description("A002", result),
            Connecting => TestStepResult::description("A003", result),
            Ui => TestStepResult::description("A004", result),
            Frequency => TestStepResult::description("A005", result),
            Pulses => TestStepResult::description("A006", result),
            Analog => TestStepResult::description("A007", result),
            Output => TestStepResult::description("A008", result),
        }
    }
}
