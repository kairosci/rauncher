use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API error: {0}")]
    Api(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
