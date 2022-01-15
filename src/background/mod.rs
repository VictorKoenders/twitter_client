mod config;
pub mod twitter;

use self::config::Config;
use crate::image;
use glium::glutin::event_loop::EventLoopProxy;
use lazy_static::lazy_static;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender,
};

const TARGET: &str = "Background";

lazy_static! {
    static ref SENDER: Mutex<Option<Sender<ToBackground>>> = Mutex::default();
}

pub fn spawn(proxy: EventLoopProxy<ToUI>) -> Background {
    // let (to_ui, from_ui) = unbounded_channel::<ToUI>();
    let (to_backend, from_backend) = unbounded_channel::<ToBackground>();
    *SENDER.lock().unwrap() = Some(to_backend.clone());
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .thread_name("Twitter background thread")
            .build()
            .unwrap();
        runtime.block_on(async move {
            let runner = Runner::new(proxy.clone(), from_backend).await;
            if let Err(e) = runner.run().await {
                log::error!(target: TARGET, "Crashed: {:?}", e);
            }
            proxy.send_event(ToUI::Disconnect).unwrap();
        })
    });
    Background { sender: to_backend }
}

pub struct Background {
    #[allow(dead_code)]
    sender: Sender<ToBackground>,
}

impl Background {
    fn send(&self, msg: ToBackground) {
        if let Err(e) = self.sender.send(msg) {
            log::warn!(
                target: TARGET,
                "Could not send message to background thread: {:?}",
                e
            )
        }
    }

    pub fn open_twitter_login(&self) {
        self.send(ToBackground::OpenTwitterLogin);
    }

    pub fn enter_twitter_pin(&self, pin: String) {
        self.send(ToBackground::TwitterPin { pin });
    }

    pub fn load_homepage(&self) {
        self.send(ToBackground::LoadInitialTweets);
    }
    pub fn load_newer(&self) {
        self.send(ToBackground::LoadNewerTweets);
    }
    pub fn load_older(&self) {
        self.send(ToBackground::LoadOlderTweets);
    }
    pub fn set_latest_tweet(&self, id: u64) {
        self.send(ToBackground::SetLatestTweet { id });
    }

    pub fn logout(&self) {}
}

struct Runner {
    sender: EventLoopProxy<ToUI>,
    receiver: Receiver<ToBackground>,
    running: bool,
    config: Config,
    state: BackgroundState,
}

impl Runner {
    async fn new(sender: EventLoopProxy<ToUI>, receiver: Receiver<ToBackground>) -> Self {
        let config = Config::load();
        let mut result = Self {
            sender,
            receiver,
            running: true,
            config,
            state: BackgroundState::NotLoggedIn,
        };
        if let Some(token) = result.config.twitter.get_token() {
            result.login_from_token(token).await;
        }
        result
    }

    async fn login_from_token(&mut self, token: egg_mode::Token) {
        self.send_to_ui(ToUI::Loading);
        self.handle_login_result(twitter::User::login_with_token(token).await);
    }

    async fn run(mut self) -> Result<(), ()> {
        while self.running {
            let timeout = tokio::time::sleep(Duration::from_secs(1));

            tokio::select! {
                msg = self.receiver.recv() => self.handle_recv(msg).await,
                _ = timeout => self.ping(),
            }
        }
        Ok(())
    }

    fn send_to_ui(&mut self, msg: ToUI) {
        if let Err(e) = self.sender.send_event(msg) {
            log::warn!(target: TARGET, "Could not send message to ui: {:?}", e);
            self.running = false;
        }
    }

    fn ping(&mut self) {
        self.send_to_ui(ToUI::Ping);
    }

    async fn handle_recv(&mut self, recv: Option<ToBackground>) {
        let msg = if let Some(msg) = recv {
            msg
        } else {
            log::info!(target: TARGET, "Receiver returned `None`, exiting");
            self.running = false;
            return;
        };
        match msg {
            ToBackground::OpenTwitterLogin => self.open_twitter_login().await,
            ToBackground::TwitterPin { pin } => self.login(pin).await,
            ToBackground::LoadInitialTweets => self.load_tweets(|t| t.start()).await,
            ToBackground::LoadOlderTweets => self.load_tweets(|t| t.older(None)).await,
            ToBackground::LoadNewerTweets => self.load_tweets(|t| t.newer(None)).await,
            ToBackground::LoadImage { key, context } => self.load_image(key, context),
            ToBackground::SetLatestTweet { id } => {
                self.config.twitter.latest = Some(id);
                self.config.save();
            }
        }
    }

    fn load_image(&self, key: image::Key, context: Arc<image::LoadContext>) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Some(result) = image::load_image_async(key, context).await {
                let _ = sender.send_event(ToUI::ImageLoaded(result));
            }
        });
    }

    async fn open_twitter_login(&mut self) {
        let request = twitter::AuthRequest::new().await;
        open::that(request.url()).unwrap();
        self.state = BackgroundState::Authing(request);
    }

    async fn load_tweets<F>(&mut self, f: F)
    where
        F: FnOnce(egg_mode::tweet::Timeline) -> egg_mode::tweet::TimelineFuture,
    {
        let timeline = if let BackgroundState::LoggedIn { timeline, .. } = &mut self.state {
            timeline.take().unwrap()
        } else {
            log::warn!(target: TARGET, "Could not load tweets; not logged in");
            return;
        };
        let future = f(timeline.with_page_size(50));
        match future.await {
            Ok((new_timeline, tweets)) => {
                log::info!(target: TARGET, "Loaded {} tweets", tweets.len());
                if let BackgroundState::LoggedIn { timeline, .. } = &mut self.state {
                    *timeline = Some(new_timeline);
                    self.send_to_ui(ToUI::Tweets {
                        tweets: tweets.response,
                        latest: self.config.twitter.latest,
                    });
                } else {
                    log::warn!(target: TARGET, "Loaded tweets but we're logged out now");
                }
            }
            Err(e) => {
                log::warn!(target: TARGET, "Could not load tweets: {:?}", e);
                self.send_to_ui(ToUI::Error {
                    error: e.to_string(),
                });
            }
        }
    }

    async fn login(&mut self, pin: String) {
        let request = match &self.state {
            BackgroundState::Authing(request) => request.clone(),
            _ => {
                log::warn!(target: TARGET, "Not in authing state, ignoring login");
                return;
            }
        };
        self.handle_login_result(request.authenticate(pin).await);
    }

    fn handle_login_result(&mut self, user: egg_mode::error::Result<twitter::User>) {
        match user {
            Ok(user) => {
                self.config.twitter.set_token(&user.token);
                self.config.save();

                self.state = BackgroundState::LoggedIn {
                    // user: user.clone(),
                    timeline: Some(egg_mode::tweet::home_timeline(&user.token)),
                };
                self.send_to_ui(ToUI::LoggedIn { user });
            }
            Err(e) => {
                self.send_to_ui(ToUI::Error {
                    error: e.to_string(),
                });
            }
        }
    }
}

enum BackgroundState {
    NotLoggedIn,
    Authing(twitter::AuthRequest),
    LoggedIn {
        // user: twitter::User,
        timeline: Option<egg_mode::tweet::Timeline>,
    },
}

pub fn start_loading_image(key: image::Key, context: Arc<image::LoadContext>) {
    let lock = SENDER.lock().unwrap();
    if let Some(sender) = lock.as_ref() {
        let _ = sender.send(ToBackground::LoadImage { key, context });
    }
}

#[derive(Debug)]
enum ToBackground {
    OpenTwitterLogin,
    TwitterPin {
        pin: String,
    },
    LoadInitialTweets,
    LoadOlderTweets,
    LoadNewerTweets,
    LoadImage {
        key: image::Key,
        context: Arc<image::LoadContext>,
    },
    SetLatestTweet {
        id: u64,
    },
}

#[derive(Debug)]
pub enum ToUI {
    Ping,
    Disconnect,
    Loading,
    LoggedIn {
        user: twitter::User,
    },
    Error {
        error: String,
    },
    Tweets {
        tweets: Vec<egg_mode::tweet::Tweet>,
        latest: Option<u64>,
    },
    ImageLoaded(image::ToUIImage),
}
