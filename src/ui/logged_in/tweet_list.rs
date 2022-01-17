use super::LoggedIn;
use crate::background::Background;
use crate::ui::utils::ClickableLink;
use egui::*;

pub fn tweet_list(state: &mut LoggedIn, background: &mut Background, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        if ui
            .add_enabled(!state.loading_more, Button::new("Load newer"))
            .clicked()
        {
            background.load_newer();
            state.loading_more = true;
        }
        let mut tweet_clicked = None;
        for tweet in state.tweets.iter().rev() {
            ui.separator();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(user) = tweet.user.as_ref() {
                        ui.add(Label::new(RichText::new(&user.name).strong()));
                        ui.separator();
                        ui.hyperlink_to(
                            "original",
                            format!(
                                "https://twitter.com/{}/status/{}",
                                user.screen_name, tweet.id
                            ),
                        );
                        ui.separator();
                        ui.hyperlink_to(
                            "user profile",
                            format!("https://twitter.com/{}", user.screen_name),
                        );
                        ui.separator();
                        if ui.add(ClickableLink::new("debug")).clicked() {
                            state.debug_tweet = Some(tweet.clone());
                        }
                    } else {
                        ui.add(Label::new(RichText::new("Could not load user").strong()));
                    }
                });
                ui.label(&tweet.text);

                let mut rect = ui.min_rect();
                rect.set_width(ui.max_rect().width());
                let rect = rect.expand(5.0);

                let is_hovered = ui.rect_contains_pointer(rect);
                let is_active = state.expanded_tweet.as_ref().map(|t| t.id) == Some(tweet.id);
                if is_hovered || is_active {
                    ui.painter()
                        .rect_filled(rect, 0., Color32::from_white_alpha(10));
                }
                if is_hovered {
                    ui.ctx().output().cursor_icon = CursorIcon::PointingHand;
                    if ui.ctx().input().pointer.any_click() {
                        tweet_clicked = Some(tweet.clone());
                    }
                }
            });
        }
        ui.separator();
        if ui
            .add_enabled(!state.loading_more, Button::new("Load older"))
            .clicked()
        {
            background.load_older();
            state.loading_more = true;
        }

        if let Some(tweet) = tweet_clicked {
            state.set_expanded_tweet(background, tweet.clone());
            use std::io::Write;
            std::fs::File::create("tweet.txt")
                .unwrap()
                .write_all(format!("{:#?}", tweet).as_bytes())
                .unwrap();
        }
    });
}
