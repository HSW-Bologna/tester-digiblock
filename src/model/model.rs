#[derive(Clone, Copy, Default, Debug)]
pub enum RgbLight {
    #[default]
    White,
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone)]
pub enum DigiblockResult<T> {
    Waiting,
    Ok,
    InvalidValue(T, T),
    CommunicationError,
}

#[derive(Debug, Clone, Default)]
pub struct DigiblockState {
    pub left_button: bool,
    pub right_button: bool,
    pub frequency: u16,
    pub pulses: u16,
    pub ma420: u16,
}

#[derive(Clone, Default)]
pub enum TestStep {
    #[default]
    Stopped,
    InvertPower(Option<String>),
    FlashingTest(Option<String>),
    Connecting(Option<String>),
    Ui {
        light: RgbLight,
    },
    Frequency(u32, bool),
    Pulses(u32, Option<Result<u16, ()>>),
    Analog(DigiblockResult<u16>),
    Output(DigiblockResult<()>),
}

#[derive(Clone, Default)]
pub struct Model {
    pub step: TestStep,
    pub digiblock_state: DigiblockState,
    pub logs: Vec<String>,
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
        match self.step {
            TestStep::Connecting(_) => {
                self.step = TestStep::Ui {
                    light: RgbLight::default(),
                }
            }
            _ => (),
        }
        self.digiblock_state = state;
    }

    pub fn next_light(&mut self) -> RgbLight {
        match self.step {
            TestStep::Ui { light } => {
                let next = match light {
                    RgbLight::White => RgbLight::Red,
                    RgbLight::Red => RgbLight::Green,
                    RgbLight::Green => RgbLight::Blue,
                    RgbLight::Blue => RgbLight::White,
                };
                self.step = TestStep::Ui { light: next };
                light
            }
            _ => RgbLight::White,
        }
    }
}
