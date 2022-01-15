use crate::background::ToUI;
use egui::*;

#[derive(Default)]
pub struct LoggedOut {
    pin: String,
    error: Option<String>,
    loading: bool,
}

impl LoggedOut {
    pub fn with_error(e: String) -> Self {
        Self {
            error: Some(e),
            ..Default::default()
        }
    }

    pub fn update(&mut self, msg: ToUI) {
        match msg {
            ToUI::Error { error } => {
                self.error = Some(error);
            }
            ToUI::Loading => {}
            x => log::warn!(target: "UI", "Ignoring {:?}", x),
        }
    }

    pub fn draw(&mut self, ctx: &mut crate::Context) {
        CentralPanel::default().show(ctx.ctx, |ui| {
            if ui
                .add_enabled(!self.loading, Button::new("Login (opens in browser)"))
                .clicked()
            {
                ctx.background.open_twitter_login();
            }
            ui.horizontal(|ui| {
                ui.label("Pin: ");
                ui.text_edit_singleline(&mut self.pin);
                if ui
                    .add_enabled(
                        !self.pin.is_empty() && !self.loading,
                        Button::new("Enter pin"),
                    )
                    .clicked()
                {
                    self.error = None;
                    self.loading = true;
                    ctx.background.enter_twitter_pin(self.pin.clone());
                }
            });
            if let Some(error) = &self.error {
                ui.label(error);
            }
        });
    }
}
