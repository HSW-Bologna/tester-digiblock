#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod controller;
mod model;
mod view;

use iced;
use iced::Application;

fn main() -> iced::Result {
    view::app::App::run(iced::Settings::default())
}
