use std::{
    fs::File,
    io::{Read, Write},
};

const TARGET: &str = "config";

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct Config {
    pub twitter: TwitterConfig,
}

impl Config {
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(config) => config,
            Err(e) => {
                log::warn!(
                    target: TARGET,
                    "Config not found, creating a default one ({:?})",
                    e
                );
                let config = Config::default();
                config.save();
                config
            }
        }
    }

    fn try_load() -> Result<Self, Error> {
        let mut file = File::open("config.toml")?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        toml::from_str(&content).map_err(Into::into)
    }

    pub fn save(&self) {
        let str = toml::to_string_pretty(&self).expect("Could not serialize config");
        File::create("config.toml")
            .unwrap()
            .write_all(str.as_bytes())
            .unwrap();
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct TwitterConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub access_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub bearer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub latest: Option<u64>,
}

impl TwitterConfig {
    pub fn clear_token(&mut self) {
        self.access_key = None;
        self.access_secret = None;
        self.bearer = None;
    }

    pub fn set_token(&mut self, token: &egg_mode::Token) {
        self.clear_token();
        match token {
            egg_mode::Token::Access { access, .. } => {
                self.access_key = Some(access.key.to_string());
                self.access_secret = Some(access.secret.to_string());
            }
            egg_mode::Token::Bearer(string) => {
                self.bearer = Some(string.clone());
            }
        }
    }

    pub fn get_token(&self) -> Option<egg_mode::Token> {
        if let (Some(key), Some(secret)) = (&self.access_key, &self.access_secret) {
            Some(egg_mode::Token::Access {
                access: egg_mode::KeyPair {
                    key: key.clone().into(),
                    secret: secret.clone().into(),
                },
                consumer: super::twitter::CONSUMER.clone(),
            })
        } else {
            self.bearer.clone().map(egg_mode::Token::Bearer)
        }
    }
}

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl From<std::io::Error> for Error {
    fn from(io: std::io::Error) -> Self {
        Self::Io(io)
    }
}
impl From<toml::de::Error> for Error {
    fn from(toml: toml::de::Error) -> Self {
        Self::Toml(toml)
    }
}
