use egg_mode::tweet::Tweet;
use egui::*;

pub fn render_tweet_debug(ui: &mut Ui, tweet: &Tweet) {
    Grid::new("tweet_debug").show(ui, |ui| {
        macro_rules! attr {
			(
				$ui:expr;
				$(
					$start:ident $(.$part:ident)*
				,)*) => {{
				$(
					$ui.label(stringify!($($part )*));
					$ui.label(format!("{:?}", $start $(.$part)*));
					$ui.end_row();
				)*
			}}
		}

        attr! {
            ui;
            tweet.id,
        }

        ui.label("user");
        if let Some(user) = tweet.user.as_ref() {
            Grid::new("tweet_debug_user").show(ui, |ui| {
                attr! {
                    ui;
                    user.id,
                    user.screen_name,
                    user.name,
                    user.verified,
                    user.protected,
                    user.description,
                    user.location,
                    user.url,
                    user.statuses_count,
                    user.friends_count,
                    user.followers_count,
                    user.favourites_count,
                    user.listed_count,
                    user.profile_image_url_https,
                    user.profile_image_url,
                }
            });
        } else {
            ui.label("None");
        }
        ui.end_row();

        attr! {
            ui;
            tweet.coordinates,
            tweet.created_at,
            tweet.current_user_retweet,
            tweet.display_text_range,
            tweet.entities.hashtags,
            tweet.entities.symbols,
            tweet.entities.urls,
            tweet.entities.user_mentions,
            tweet.entities.media,
            tweet.favorite_count,
            tweet.favorited,
            tweet.filter_level,
            tweet.in_reply_to_user_id,
            tweet.in_reply_to_screen_name,
            tweet.in_reply_to_status_id,
            tweet.lang,
            tweet.place,
            tweet.possibly_sensitive,
            tweet.quoted_status_id,
            tweet.retweet_count,
            tweet.retweeted,
            tweet.truncated,
            tweet.withheld_copyright,
            tweet.withheld_in_countries,
            tweet.withheld_scope,
        }
    });
}
