use reqwest::{self, header::InvalidHeaderValue};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("InvalidHeader error: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),

    #[error("Request error: {0}")]
    InvalidRequest(String),

    #[error("Request id: {request_id}, Error code: {error_code}, Error message: {error_message}")]
    InvalidResponse {
        request_id: String,
        error_code: String,
        error_message: String,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
