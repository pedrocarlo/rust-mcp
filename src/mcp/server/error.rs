use axum_derive_error::ErrorResponse;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Error, ErrorResponse)]
pub enum ApiError {
    // Cannot use from PoisonedError here as it requires a generic param
    #[error("Poisoned Lock")]
    PoisonedLock,
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
    #[error("Missing Client Error")]
    MissingClient,
}
