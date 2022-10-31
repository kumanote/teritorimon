pub mod account;
mod error;

pub type Result<T> = anyhow::Result<T>;
pub use error::Error;
