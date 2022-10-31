use crate::Result;
use anyhow::Context;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct ApplicationToml {
    pub interval: Option<String>,
    pub checkers: Vec<CheckerToml>,
    pub logger: Option<LoggerToml>,
}

impl ApplicationToml {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(&path).with_context(|| {
            format!(
                "could not open toml file of {:?}",
                path.as_ref().as_os_str()
            )
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).with_context(|| {
            format!(
                "could not read toml file of {:?}",
                path.as_ref().as_os_str()
            )
        })?;
        toml::from_str(contents.as_str()).with_context(|| {
            format!(
                "could not parse toml file of {:?}",
                path.as_ref().as_os_str()
            )
        })
    }
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct CheckerToml {
    pub teritori_grpc_scheme: Option<String>,
    pub teritori_grpc_host: Option<String>,
    pub teritori_grpc_port: Option<u16>,
    pub validator_account: Option<String>,
    pub validator_address: Option<String>,
    pub syncing: Option<bool>,
    pub new_proposal: Option<bool>,
    pub missed_block: Option<bool>,
    pub missed_block_threshold: Option<String>,
    pub validator_status: Option<bool>,
    pub slashes: Option<bool>,
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub struct LoggerToml {
    pub chan_size: Option<usize>,
    pub is_async: Option<bool>,
    pub level: Option<String>,
    pub airbrake_host: Option<String>,
    pub airbrake_project_id: Option<String>,
    pub airbrake_project_key: Option<String>,
    pub airbrake_environment: Option<String>,
}
