#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod controller;
mod model;
mod view;

use controller::reles::Rele;
use iced;
use iced::Application;

fn main() -> iced::Result {
    //controller::reles::set_reles().ok();

    controller::reles::update(Rele::Enable420ma, true).ok();
    controller::reles::update(Rele::ShortCircuit, false).ok();

    let res = controller::pwm::set_pwm();
    println!("{:?}", res);

    //controller::reles::update(Rele::Enable420ma, false).ok();

    controller::app::App::run(iced::Settings::default())
}
