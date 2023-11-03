use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")] HttpRequestFailure(#[from] reqwest::Error),
    #[error("JSON parsing failed: {0}")] JsonParsingFailure(#[from] serde_json::Error),
    #[error("Invalid HTTP header value: {0}")] InvalidHttpHeaderValue(
        #[from] reqwest::header::InvalidHeaderValue,
    ),
    #[error("Input/Output operation failed: {0}")] IoOperationFailure(#[from] std::io::Error),
}
