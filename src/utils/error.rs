use std::fmt;

#[derive(Debug)]
pub enum KonaError {
    ApiError(String),
    ConfigError(String),
    IoError(std::io::Error),
}

impl fmt::Display for KonaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KonaError::ApiError(msg) => write!(f, "API Error: {}", msg),
            KonaError::ConfigError(msg) => write!(f, "Config Error: {}", msg),
            KonaError::IoError(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl std::error::Error for KonaError {}

impl From<std::io::Error> for KonaError {
    fn from(error: std::io::Error) -> Self {
        KonaError::IoError(error)
    }
}

pub type Result<T> = std::result::Result<T, KonaError>;