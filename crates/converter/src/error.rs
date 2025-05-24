use thiserror::Error;

#[derive(Error, Debug)]
pub enum DetectionError {
    #[error("JSON decode error")]
    JsonDecodeError(#[from] serde_json::Error),

    #[error("Format could not be detected")]
    UnknownFormat,
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Unknown ID in input: {0}")]
    UnknownId(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
