use crate::Error;
use std::fmt;
use std::str::FromStr;
use subtle_encoding::bech32;
use subtle_encoding::hex;

pub const LENGTH: usize = 20;

#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Id([u8; LENGTH]);

impl Id {
    /// Create a new account ID from raw bytes
    pub fn new(bytes: [u8; LENGTH]) -> Id {
        Id(bytes)
    }

    /// Borrow the account ID as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }

    pub fn get_bech32_address_string(&self, prefix: &str) -> String {
        bech32::encode(prefix, self.as_bytes())
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

impl TryFrom<Vec<u8>> for Id {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != LENGTH {
            return Err(Error::InvalidAccountIdLength);
        }
        let mut slice: [u8; LENGTH] = [0; LENGTH];
        slice.copy_from_slice(&value[..]);
        Ok(Id(slice))
    }
}

/// Decode account ID from hex
impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode_upper(s)
            .or_else(|_| hex::decode(s))
            .map_err(|_| Error::InvalidHexAddress)?;
        bytes.try_into()
    }
}
