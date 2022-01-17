use crate::ui::utils::Image;
use egg_mode::tweet::Tweet;
use egui::*;

pub fn draw_tweet(ctx: &mut crate::Context, ui: &mut Ui, tweet: &Tweet) {
    let user = tweet.user.as_ref().unwrap();
    ui.horizontal(|ui| {
        ui.add(Image::https(&user.profile_image_url_https, (48., 48.)));
        ui.vertical(|ui| {
            ui.label(RichText::new(&user.name).strong());
            ui.horizontal(|ui| {
                ui.hyperlink_to(
                    format!("@{}", user.screen_name),
                    format!("https://twitter.com/{}", user.screen_name),
                );
                ui.separator();
                ui.hyperlink_to(
                    "original",
                    format!(
                        "https://twitter.com/{}/status/{}",
                        user.screen_name, tweet.id
                    ),
                );
            });
        });
    });
    if let Some(description) = user.description.as_ref() {
        ui.label(description);
    }
    ui.separator();

    if let Some(nested) = &tweet.retweeted_status {
        ui.label(RichText::new("Retweeted:").strong());
        ui.separator();
        draw_tweet(ctx, ui, nested);
    } else {
        ui.label(RichText::new(tweet.text.as_str()).strong());
        let max = ui.max_rect().size().min_elem();

        // ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal(|ui| {
            let media = tweet
                .extended_entities
                .as_ref()
                .map(|e| e.media.as_slice())
                .or_else(|| tweet.entities.media.as_deref())
                .unwrap_or_default();
            for media in media {
                let size = media.sizes.large;
                let mut size = Vec2::from((size.w as f32, size.h as f32));
                if size.max_elem() > max {
                    let scale = max / size.max_elem();
                    size *= scale;
                }

                ui.add(Image::https(&media.media_url_https, size));
            }
        });
        // });

        if let Some(quoted) = &tweet.quoted_status {
            draw_tweet(ctx, ui, quoted);
        }
    }
}
