use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid hex address error")]
    InvalidHexAddress,
    #[error("invalid account ID length")]
    InvalidAccountIdLength,
}
