use crate::{CustomError, Result};
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub fn calculate_hash(bytes: &[u8]) -> Result<String> {
    // sha256
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let output = hasher.finalize();
    let mut result = String::new();
    for byte in &output[..32] {
        write!(result, "{:02X?}", byte).map_err(|cause| CustomError::from(cause))?;
    }
    Ok(result)
}
