use crate::background::{twitter::User, Background, ToUI};
use egg_mode::tweet::Tweet;
use egui::*;
use std::collections::VecDeque;
use winit::event::VirtualKeyCode;

pub struct LoggedIn {
    user: User,
    error: Option<String>,
    tweets: VecDeque<Tweet>,
    debug_tweet: Option<Tweet>,
    expanded_tweet: Option<Tweet>,
    loading_more: bool,
}

impl LoggedIn {
    pub fn new(user: User, background: &mut Background) -> Box<Self> {
        background.load_homepage();
        Box::new(Self {
            user,
            error: None,
            tweets: VecDeque::new(),
            debug_tweet: None,
            expanded_tweet: None,
            loading_more: false,
        })
    }

    fn set_expanded_tweet(&mut self, background: &mut Background, tweet: Tweet) {
        background.set_latest_tweet(tweet.id);
        self.expanded_tweet = Some(tweet);
    }

    pub fn update(&mut self, background: &mut Background, msg: ToUI) {
        match msg {
            ToUI::Error { error } => {
                self.error = Some(error);
            }
            ToUI::Loading => {}
            ToUI::Tweets { tweets, latest } => {
                for tweet in tweets {
                    match self.tweets.binary_search_by_key(&tweet.id, |t| t.id) {
                        Ok(idx) => self.tweets[idx] = tweet,
                        Err(idx) => self.tweets.insert(idx, tweet),
                    }
                }
                self.loading_more = false;
                if self.expanded_tweet.is_none() {
                    if let Some(latest) = latest {
                        if let Some(tweet) = self.tweets.iter().find(|t| t.id == latest).cloned() {
                            self.set_expanded_tweet(background, tweet);
                        }
                    }
                }
            }
            x => log::warn!(target: "UI", "Ignoring {:?}", x),
        }
    }

    pub fn draw(&mut self, ctx: &mut crate::Context) {
        if let Some(tweet) = &self.debug_tweet {
            let mut open = true;
            Window::new("Debug tweet")
                .hscroll(true)
                .vscroll(true)
                .open(&mut open)
                .show(ctx.ctx, |ui| {
                    render_tweet_debug(ui, tweet);
                })
                .unwrap();
            if !open {
                self.debug_tweet = None;
            }
        }
        SidePanel::left("tweet_list").show(ctx.ctx, |ui| {
            self.tweet_list(ctx.background, ui);
        });
        if let Some(error) = &self.error {
            TopBottomPanel::top("tweet_error").show(ctx.ctx, |ui| {
                ui.label(error);
            });
        }
        if let Some(tweet) = &self.expanded_tweet {
            CentralPanel::default().show(ctx.ctx, |ui| {
                draw_tweet(ctx, ui, tweet);
            });
        }
    }

    fn tweet_list(&mut self, background: &mut Background, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(RichText::new(&self.user.name).strong()));
                if ui.add(ClickableLink::new("log out")).clicked() {
                    background.logout();
                }
            });
            ui.separator();
            if ui
                .add_enabled(!self.loading_more, Button::new("Load newer"))
                .clicked()
            {
                background.load_newer();
                self.loading_more = true;
            }
            let mut tweet_clicked = None;
            for tweet in self.tweets.iter().rev() {
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
                                self.debug_tweet = Some(tweet.clone());
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
                    let is_active = self.expanded_tweet.as_ref().map(|t| t.id) == Some(tweet.id);
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
                .add_enabled(!self.loading_more, Button::new("Load older"))
                .clicked()
            {
                background.load_older();
                self.loading_more = true;
            }

            if let Some(tweet) = tweet_clicked {
                self.set_expanded_tweet(background, tweet.clone());
                use std::io::Write;
                std::fs::File::create("tweet.txt")
                    .unwrap()
                    .write_all(format!("{:#?}", tweet).as_bytes())
                    .unwrap();
            }
        });
    }

    pub fn key_pressed(&mut self, background: &mut Background, keycode: VirtualKeyCode) {
        match keycode {
            VirtualKeyCode::Up => {
                if let Some(tweet) = &self.expanded_tweet {
                    if let Some(idx) = self.tweets.iter().position(|t| t.id == tweet.id) {
                        if let Some(tweet) = self.tweets.get(idx + 1).cloned() {
                            self.set_expanded_tweet(background, tweet);
                        } else {
                            background.load_newer();
                            self.loading_more = true;
                        }
                    }
                }
            }
            VirtualKeyCode::Down => {
                if let Some(tweet) = &self.expanded_tweet {
                    if let Some(idx) = self.tweets.iter().position(|t| t.id == tweet.id) {
                        if idx > 0 {
                            let tweet = self.tweets.get(idx - 1).unwrap().clone();
                            self.set_expanded_tweet(background, tweet);
                        } else {
                            background.load_older();
                            self.loading_more = true;
                        }
                    }
                }
            }
            VirtualKeyCode::Home => {
                if let Some(last) = self.tweets.back().cloned() {
                    self.set_expanded_tweet(background, last);
                }
            }
            VirtualKeyCode::F5 => {
                background.load_newer();
                self.loading_more = true;
            }
            _ => {}
        }
    }
}

fn render_tweet_debug(ui: &mut Ui, tweet: &Tweet) {
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

struct ClickableLink {
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

// fn bigger_profile_url(url: &String) -> String {
//     if let Some((url, extension)) = url.rsplit_once('.') {
//         if let Some(base) = url.strip_suffix("_normal") {
//             return format!("{}_bigger.{}", base, extension);
//         }
//     }
//     url.clone()
// }

fn draw_tweet(ctx: &mut crate::Context, ui: &mut Ui, tweet: &Tweet) {
    let user = tweet.user.as_ref().unwrap();
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.add(super::image::Image::https(
                &user.profile_image_url_https,
                (48., 48.),
            ));
            ui.vertical(|ui| {
                ui.label(RichText::new(&user.name).strong());
                ui.hyperlink_to(
                    format!("@{}", user.screen_name),
                    format!("https://twitter.com/{}", user.screen_name),
                );
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

            ScrollArea::horizontal().show(ui, |ui| {
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
                        if size.y > 300. {
                            let scale = 300. / size.y;
                            size *= scale;
                        }

                        ui.add(super::image::Image::https(&media.media_url_https, size));
                    }
                })
            });
        }
    });
}

// struct TweetText<'a> {
//     text: Vec<TweetTextContent>,
//     media: Vec<&'a egg_mode::entities::MediaEntity>,
//     nested: Option<Box<TweetText<'a>>>,
//     profile_url: String,
//     user_name: String,
//     screen_name: String,
//     user_description: String,
// }

// #[derive(Clone)]
// enum TweetTextContent {
//     Text(String),
//     User { name: String, url: String },
// }

// impl<'a> TweetText<'a> {
//     fn new(tweet: &'a Tweet) -> Self {
//         let mut text = tweet.text.clone();
//         let mut media = Vec::new();
//         for item in tweet
//             .extended_entities
//             .iter()
//             .map(|e| e.media.iter())
//             .flatten()
//         {
//             media.push(item);
//             text = text.as_str().replace(item.url.as_str(), "");
//         }
//         let mut text = vec![TweetTextContent::Text(text)];
//         for mention in tweet.entities.user_mentions.iter() {
//             let search = format!("@{}", mention.screen_name);
//             let user = TweetTextContent::User {
//                 name: mention.name.clone(),
//                 url: format!("https://twitter.com/{}", mention.screen_name),
//             };
//             for i in (0..text.len()).rev() {
//                 if let TweetTextContent::Text(s) = text[i].clone() {
//                     let split = split_inclusive(&s, &search)
//                         .into_iter()
//                         .map(|s| {
//                             if s == search {
//                                 user.clone()
//                             } else {
//                                 TweetTextContent::Text(s.to_string())
//                             }
//                         })
//                         .collect::<Vec<_>>();
//                     if split.len() > 1 {
//                         text.remove(i);
//                         for (offset, elem) in split.into_iter().enumerate() {
//                             text.insert(i + offset, elem);
//                         }
//                     }
//                 }
//             }
//         }

//         let nested = if let Some(tweet) = tweet.retweeted_status.as_ref() {
//             Some(Box::new(TweetText::new(tweet)))
//         } else {
//             None
//         };
//         let user = tweet.user.as_ref().unwrap();

//         Self {
//             text,
//             media,
//             nested,
//             profile_url: bigger_profile_url(&user.profile_image_url_https),
//             screen_name: user.screen_name.clone(),
//             user_description: user.description.clone().unwrap_or_default(),
//             user_name: user.name.clone(),
//         }
//     }

//     fn draw_text(text: &TweetTextContent, ui: &mut Ui) {}
// }

// impl Widget for TweetText<'_> {
//     fn ui(self, ui: &mut Ui) -> Response {
//         ui.vertical(|ui| {
//             ui.horizontal(|ui| {
//                 ui.add(super::image::Image::https(self.profile_url, (48., 48.)));
//                 ui.vertical(|ui| {
//                     ui.label(RichText::new(&self.user_name).strong());
//                     ui.hyperlink_to(
//                         format!("@{}", self.screen_name),
//                         format!("https://twitter.com/{}", self.screen_name),
//                     );
//                 });
//             });
//             ui.label(self.user_description);
//             ui.separator();

//             ui.with_layout(Layout::left_to_right().with_cross_align(Align::Min), |ui| {
//                 for text in self.text {
//                     match text {
//                         TweetTextContent::Text(s) => {
//                             ui.label(RichText::new(s).strong());
//                         }
//                         TweetTextContent::User { name, url } => {
//                             ui.hyperlink_to(name, url);
//                         }
//                     }
//                 }
//             });
//             ScrollArea::horizontal().show(ui, |ui| {
//                 ui.horizontal(|ui| {
//                     for media in &self.media {
//                         let size = media.sizes.large;
//                         let mut size = Vec2::from((size.w as f32, size.h as f32));
//                         if size.y > 300. {
//                             let scale = 300. / size.y;
//                             size *= scale;
//                         }

//                         ui.add(super::image::Image::https(&media.media_url_https, size));
//                     }
//                 })
//             });

//             if let Some(nested) = self.nested {
//                 ui.separator();
//                 nested.ui(ui);
//             }
//         })
//         .response
//     }
// }

// fn split_inclusive<'a>(str: &'a String, search: &str) -> impl IntoIterator<Item = &'a str> {
//     let mut i = 0;
//     let mut result = Vec::new();
//     while let Some(idx) = str.get(i..).and_then(|s| s.find(search)) {
//         if idx > 0 {
//             result.push(&str[i..i + idx]);
//         }
//         result.push(&str[i + idx..i + idx + search.len()]);
//         i += idx + search.len();
//     }
//     if i != search.len() {
//         result.push(&str[i..]);
//     }
//     result
// }

// #[test]
// fn test_split_inclusive() {
//     assert_eq!(
//         split_inclusive(&String::from("Foo @bar @bar baz"), "@bar")
//             .into_iter()
//             .collect::<Vec<_>>(),
//         vec!["Foo ", "@bar", " ", "@bar", " baz"]
//     );
// }
