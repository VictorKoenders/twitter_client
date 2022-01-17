mod render_tweet_debug;
mod tweet;
mod tweet_list;

use super::utils::*;
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
                    render_tweet_debug::render_tweet_debug(ui, tweet);
                })
                .unwrap();
            if !open {
                self.debug_tweet = None;
            }
        }
        SidePanel::left("tweet_list").show(ctx.ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(Label::new(RichText::new(&self.user.name).strong()));
                if ui.add(ClickableLink::new("log out")).clicked() {
                    ctx.background.logout();
                }
            });
            ui.separator();
            tweet_list::tweet_list(self, ctx.background, ui);
        });
        if let Some(error) = &self.error {
            TopBottomPanel::top("tweet_error").show(ctx.ctx, |ui| {
                ui.label(error);
            });
        }
        if let Some(tweet) = &self.expanded_tweet {
            CentralPanel::default().show(ctx.ctx, |ui| {
                tweet::draw_tweet(ctx, ui, tweet);
            });
        }
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
