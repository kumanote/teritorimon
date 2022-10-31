mod error;
pub mod is_syncing;
pub mod message;
pub mod missed_block;
pub mod new_proposal;
pub mod slashes;
pub mod utils;
pub mod validator_status;

pub use error::*;
pub type Result<T> = anyhow::Result<T>;
