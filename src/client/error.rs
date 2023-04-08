use reqwest::{self, header::InvalidHeaderValue};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("InvalidHeader error: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),

    #[error("Request error: {0}")]
    Request(#[from] RequestError),

    #[error("Error code: {error_code}, Error message: {error_message}")]
    InvalidResponse {
        error_code: String,
        error_message: String,
    },
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
