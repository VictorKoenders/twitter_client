#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod background;
mod ui;

fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init();
    egui_with_background::run(ui::State::default());
}

type Context<'a> = egui_with_background::Context<'a, background::Background>;
