use tokio_modbus::client::Context;
use tokio_modbus::prelude::*;

use crate::model::{DigiblockState, RgbLight};

const INPUT_REGISTER_BTN_TEST: u16 = 0;
//const INPUT_REGISTER_FREQUENCY: u16 = 1;
//const INPUT_REGISTER_PULSES: u16 = 2;
//const INPUT_REGISTER_420MA: u16 = 3;
const NUM_INPUT_REGISTERS: u16 = 7;

const HOLDING_REGISTER_MODE: u16 = 0;
const HOLDING_REGISTER_RESET_PULSES: u16 = 1;
const HOLDING_REGISTER_OUTPUT: u16 = 2;
const HOLDING_REGISTER_BACKLIGHT: u16 = 3;
const HOLDING_REGISTER_RGB: u16 = 4;

const DIGITAL_MODE: u16 = 1;
const ANALOG_MODE: u16 = 2;

impl TryFrom<Vec<u16>> for DigiblockState {
    type Error = ();

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        if value.len() < NUM_INPUT_REGISTERS as usize {
            Err(())
        } else {
            Ok(DigiblockState {
                left_button: (value[0] & 0x01) > 0,
                right_button: (value[0] & 0x02) > 0,
                frequency: value[1],
                pulses: value[2],
                ma420: value[3],
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
        HOLDING_REGISTER_BACKLIGHT,
        &[
            100,
            match light {
                RgbLight::White => 6,
                RgbLight::Red => 7,
                RgbLight::Green => 2,
                RgbLight::Blue => 1,
            } as u16,
        ],
    )
    .await
    .map_err(|_| ())
}

pub async fn set_frequency_mode(ctx: &mut Context) -> Result<(), ()> {
    ctx.write_multiple_registers(HOLDING_REGISTER_MODE, &[DIGITAL_MODE])
        .await
        .map_err(|_| ())
}

pub async fn set_analog_mode(ctx: &mut Context) -> Result<(), ()> {
    ctx.write_multiple_registers(HOLDING_REGISTER_MODE, &[ANALOG_MODE])
        .await
        .map_err(|_| ())
}

pub async fn reset_pulses(ctx: &mut Context) -> Result<(), ()> {
    ctx.write_multiple_registers(HOLDING_REGISTER_RESET_PULSES, &[1])
        .await
        .map_err(|_| ())
}

pub async fn set_output(ctx: &mut Context, value: bool) -> Result<(), ()> {
    ctx.write_multiple_registers(HOLDING_REGISTER_OUTPUT, &[if value { 1 } else { 0 }])
        .await
        .map_err(|_| ())
}
