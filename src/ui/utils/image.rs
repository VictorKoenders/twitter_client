use egui::{Vec2, Widget};
use std::sync::Arc;

use crate::image::{self, Key, LoadContext};

pub struct Image {
    context: Arc<LoadContext>,
    size: Vec2,
}

impl Image {
    pub fn https(url: impl Into<String>, size: impl Into<Vec2>) -> Self {
        let context = image::get_context(Key::Https(url.into()));
        Self {
            context,
            size: size.into(),
        }
    }
}

impl Widget for Image {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if let Some(id) = self.context.get_texture_id() {
            egui::Image::new(id, self.size).ui(ui)
        } else if let Some(msg) = self.context.get_error() {
            egui::Label::new(msg).ui(ui)
        } else {
            egui::Label::new("").ui(ui)
        }
    }
}