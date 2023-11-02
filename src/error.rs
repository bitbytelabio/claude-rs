use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to send http request")] HttpRequestError(#[from] reqwest::Error),
    #[error("failed to parse json response")] ParseJsonError(#[from] serde_json::Error),
    #[error("can not parse http header")] ParseHttpHeaderError(
        #[from] reqwest::header::InvalidHeaderValue,
    ),
}
