use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Template error: {0}")]
    Template(String),

    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Blog post parsing error: {0}")]
    BlogParsing(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::NotFound(_) => (axum::http::StatusCode::NOT_FOUND, "Not Found"),
            AppError::Template(_) | AppError::BlogParsing(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            ),
            AppError::Config(_) | AppError::Internal(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            ),
            AppError::Io(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            ),
        };

        tracing::error!("Request error: {}", self);

        (status, error_message).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
