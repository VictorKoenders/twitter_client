pub mod utils;

mod logged_in;
mod logged_out;

use self::logged_in::LoggedIn;
use self::logged_out::LoggedOut;
use crate::background::{Background, ToUI};
use egui_with_background::winit::VirtualKeyCode;

pub struct State {
    running: bool,
    state: TwitterState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            running: true,
            state: Default::default(),
        }
    }
}

impl egui_with_background::App for State {
    type Msg = ToUI;
    type Background = Background;

    fn title(&self) -> &'static str {
        "Rusty twitter client"
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn spawn_background(
        &self,
        proxy: egui_with_background::winit::EventLoopProxy<Self::Msg>,
    ) -> Self::Background {
        crate::background::spawn(proxy)
    }

    fn handle_message(&mut self, background: &mut Background, msg: ToUI) {
        match (msg, &mut self.state) {
            (ToUI::Disconnect, x) => {
                *x = TwitterState::LoggedOut(LoggedOut::with_error(String::from(
                    "Lost connection to server",
                )));
            }
            (ToUI::LoggedIn { user }, x) => {
                let logged_in = LoggedIn::new(user, background);
                *x = TwitterState::LoggedIn(logged_in);
            }
            (msg, TwitterState::LoggedOut(state)) => state.update(msg),
            (msg, TwitterState::LoggedIn(state)) => state.update(background, msg),
        }
    }

    fn key_pressed(&mut self, background: &mut Background, keycode: VirtualKeyCode) {
        if keycode == VirtualKeyCode::Escape {
            self.running = false;
        }
        if let TwitterState::LoggedIn(state) = &mut self.state {
            state.key_pressed(background, keycode);
        }
    }

    fn key_released(&mut self, _background: &mut Background, _keycode: VirtualKeyCode) {}

    fn draw(&mut self, ctx: &mut crate::Context) {
        match &mut self.state {
            TwitterState::LoggedOut(state) => state.draw(ctx),
            TwitterState::LoggedIn(state) => state.draw(ctx),
        }
    }
}

enum TwitterState {
    LoggedOut(LoggedOut),
    LoggedIn(Box<LoggedIn>),
}

impl Default for TwitterState {
    fn default() -> Self {
        Self::LoggedOut(Default::default())
    }
}
