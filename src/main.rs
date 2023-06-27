#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod controller;
mod model;
mod view;

use iced;
use iced::Application;

fn main() -> iced::Result {
    controller::adc::read_adc(controller::adc::Channel::VBat).ok();
    controller::adc::read_adc(controller::adc::Channel::PowerConsumption).ok();
    controller::adc::read_adc(controller::adc::Channel::Out1).ok();
    controller::adc::read_adc(controller::adc::Channel::VRef).ok();
    controller::adc::read_adc(controller::adc::Channel::Press).ok();
    controller::adc::read_adc(controller::adc::Channel::Volt5).ok();
    controller::adc::read_adc(controller::adc::Channel::Supply).ok();
    controller::adc::read_adc(controller::adc::Channel::Volt3).ok();

    controller::reles::all_off();

    //let res = controller::pwm::set_pwm();
    //println!("{:?}", res);

    //controller::reles::update(Rele::Enable420ma, false).ok();

    controller::app::App::run(iced::Settings::default())
}
