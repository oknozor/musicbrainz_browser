use iced::{Application, Settings};
use gui::App;

pub mod model;
mod data_providers;
mod gui;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}









