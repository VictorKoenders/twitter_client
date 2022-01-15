use egg_mode::{error::Result, *};
use std::borrow::Cow;

pub const CONSUMER: KeyPair = KeyPair {
    key: Cow::Borrowed(dotenv_codegen::dotenv!("TWITTER_CLIENT_ID")),
    secret: Cow::Borrowed(dotenv_codegen::dotenv!("TWITTER_CLIENT_SECRET")),
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
