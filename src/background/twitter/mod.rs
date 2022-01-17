use egg_mode::{error::Result, *};
use std::borrow::Cow;

// Generated in build.rs
include!(concat!(env!("OUT_DIR"), "/twitter_credentials.rs"));

pub const CONSUMER: KeyPair = KeyPair {
    key: Cow::Borrowed(twitter_id()),
    secret: Cow::Borrowed(twitter_secret()),
};

#[derive(Clone)]
pub struct AuthRequest {
    token: KeyPair,
}

impl AuthRequest {
    pub async fn new() -> Self {
        Self {
            token: auth::request_token(&CONSUMER, "oob").await.unwrap(),
        }
    }

    pub fn url(&self) -> String {
        auth::authorize_url(&self.token)
    }

    pub async fn authenticate(self, pin: String) -> Result<User> {
        let (token, id, name) = auth::access_token(CONSUMER.clone(), &self.token, pin).await?;
        Ok(User { token, id, name })
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub token: Token,
    pub id: u64,
    pub name: String,
}

impl User {
    pub async fn login_with_token(token: Token) -> Result<Self> {
        auth::verify_tokens(&token).await.map(|r| Self {
            token,
            id: r.response.id,
            name: r.response.name,
        })
    }
}
