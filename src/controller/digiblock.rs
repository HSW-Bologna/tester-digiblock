use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

use crate::model::{DigiblockState, RgbLight};

const INPUT_REGISTER_BTN_TEST: u16 = 0;
const INPUT_REGISTER_FREQUENCY: u16 = 1;
const INPUT_REGISTER_PULSES: u16 = 2;
const NUM_INPUT_REGISTERS: u16 = 7;

const HOLDING_REGISTER_RGB: u16 = 4;

impl TryFrom<Vec<u16>> for DigiblockState {
    type Error = ();

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        if value.len() < NUM_INPUT_REGISTERS as usize {
            Err(())
        } else {
            Ok(DigiblockState {
                left_button: (value[0] & 0x01) > 0,
                right_button: (value[0] & 0x02) > 0,
            })
        }
    }
}

pub async fn get_state(ctx: &mut Context) -> Result<DigiblockState, ()> {
    ctx.read_input_registers(INPUT_REGISTER_BTN_TEST, NUM_INPUT_REGISTERS)
        .await
        .map_err(|_| ())
        .and_then(|bytes| DigiblockState::try_from(bytes))
}

pub async fn set_light(ctx: &mut Context, light: RgbLight) -> Result<(), ()> {
    ctx.write_multiple_registers(
        HOLDING_REGISTER_RGB,
        &[match light {
            RgbLight::White => 6,
            RgbLight::Red => 7,
            RgbLight::Green => 2,
            RgbLight::Blue => 1,
        } as u16],
    )
    .await
    .map_err(|_| ())
}
