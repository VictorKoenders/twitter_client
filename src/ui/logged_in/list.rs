use crate::background::Background;
use egg_mode::tweet::Tweet;
use egui::*;

pub fn tweet_list<'a, ITER>(
    tweets: ITER,
    loading_more: &mut bool,
    expanded_tweet: &Option<Tweet>,
    background: &mut Background,
    ui: &mut Ui,
) -> Option<Tweet>
where
    ITER: Iterator<Item = &'a Tweet> + 'a + DoubleEndedIterator,
{
    let mut new_tweet = None;
    ScrollArea::vertical().show(ui, |ui| {
        if ui
            .add_enabled(!*loading_more, Button::new("Load newer"))
            .clicked()
        {
            background.load_newer();
            *loading_more = true;
        }
        let mut tweet_clicked = None;
        for tweet in tweets.rev() {
            ui.separator();
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(user) = tweet.user.as_ref() {
                        ui.add(Label::new(RichText::new(&user.name).strong()));
                    } else {
                        ui.add(Label::new(RichText::new("Could not load user").strong()));
                    }
                });
                ui.label(&tweet.text);

                let mut rect = ui.min_rect();
                rect.set_width(ui.max_rect().width());
                let rect = rect.expand(5.0);

                let is_hovered = ui.rect_contains_pointer(rect);
                let is_active = expanded_tweet.as_ref().map(|t| t.id) == Some(tweet.id);
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
            .add_enabled(!*loading_more, Button::new("Load older"))
            .clicked()
        {
            background.load_older();
            *loading_more = true;
        }

        if let Some(tweet) = tweet_clicked {
            new_tweet = Some(tweet.clone());
        }
    });
    new_tweet
}
