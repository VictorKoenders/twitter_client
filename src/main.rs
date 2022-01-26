#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod background;
mod ui;

fn main() {
    egui_with_background::run(ui::State::default());
}

type Context<'a> = egui_with_background::Context<'a, background::Background>;
