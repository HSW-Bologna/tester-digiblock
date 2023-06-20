#[derive(Clone, Copy, Default, Debug)]
pub enum RgbLight {
    #[default]
    White,
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Default)]
pub struct DigiblockState {
    pub left_button: bool,
    pub right_button: bool,
}

#[derive(Clone, Copy, Default)]
pub enum TestState {
    #[default]
    Unresponsive,
    Ui {
        light: RgbLight,
    },
}

#[derive(Clone, Copy, Default)]
pub enum State {
    #[default]
    Disconnected,
    Connected(TestState),
}

#[derive(Clone, Default)]
pub struct Model {
    pub state: State,
    pub digiblock_state: DigiblockState,
    pub ports: Vec<String>,
    pub selected_port: Option<String>,
    pub logs: Vec<String>,
}

impl Model {
    pub fn update_ports(&mut self, ports: Vec<String>) {
        self.ports = ports;
        if let Some(port) = &self.selected_port {
            if !self.ports.contains(&port) {
                self.selected_port = None;
            }
        } else if self.ports.len() > 0 {
            self.selected_port = Some(self.ports[0].clone());
        }
    }

    pub fn logs(&self) -> String {
        let mut logs = String::new();

        for s in &self.logs {
            logs += &String::from(s.clone() + "\n");
        }

        logs
    }

    pub fn connected(&self) -> bool {
        match self.state {
            State::Connected(_) => true,
            State::Disconnected => false,
        }
    }

    pub fn digiblock_update(&mut self, state: DigiblockState) {
        match self.state {
            State::Connected(TestState::Unresponsive) => {
                self.state = State::Connected(TestState::Ui {
                    light: RgbLight::default(),
                })
            }
            _ => (),
        }
        self.digiblock_state = state;
    }

    pub fn next_light(&mut self) -> RgbLight {
        match self.state {
            State::Connected(TestState::Ui { light }) => {
                let next = match light {
                    RgbLight::White => RgbLight::Red,
                    RgbLight::Red => RgbLight::Green,
                    RgbLight::Green => RgbLight::Blue,
                    RgbLight::Blue => RgbLight::White,
                };
                self.state = State::Connected(TestState::Ui { light: next });
                light
            }
            _ => RgbLight::White,
        }
    }
}
