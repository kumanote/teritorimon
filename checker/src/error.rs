use thiserror::Error as ThisError;
pub type Error = anyhow::Error;

#[derive(ThisError, Debug)]
pub enum CustomError {
    #[error("transcode error due to \"{reason}\"")]
    Transcode { reason: String },
    #[error("format error \"{cause}\"")]
    IllegalFormat { cause: std::fmt::Error },
}

impl From<std::fmt::Error> for CustomError {
    fn from(cause: std::fmt::Error) -> Self {
        CustomError::IllegalFormat { cause }
    }
}
