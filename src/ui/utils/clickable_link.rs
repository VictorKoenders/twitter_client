use egui::*;

pub struct ClickableLink {
    text: WidgetText,
}

impl ClickableLink {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for ClickableLink {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { text } = self;
        let label = Label::new(text).sense(Sense::click());

        let (pos, text_galley, response) = label.layout_in_ui(ui);

        if response.hovered() {
            ui.ctx().output().cursor_icon = CursorIcon::PointingHand;
        }

        if ui.is_rect_visible(response.rect) {
            let color = ui.visuals().hyperlink_color;
            let visuals = ui.style().interact(&response);

            let underline = if response.hovered() || response.has_focus() {
                Stroke::new(visuals.fg_stroke.width, color)
            } else {
                Stroke::none()
            };

            ui.painter().add(epaint::TextShape {
                pos,
                galley: text_galley.galley,
                override_text_color: Some(color),
                underline,
                angle: 0.0,
            });
        }

        response
    }
}
