use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ParseError(#[from] toml::de::Error),

    #[error(transparent)]
    FromUtf16Error(#[from] std::string::FromUtf16Error),
}

pub type Result<T> = std::result::Result<T, Error>;
