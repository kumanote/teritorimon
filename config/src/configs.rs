use crate::toml::*;
use crate::{MissedBlockThreshold, Result};
use anyhow::{anyhow, Context};
use std::env;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

const DEFAULT_INTERVAL: &'static str = "10s";

pub trait FromEnv: Sized {
    fn from_env() -> Result<Self>;
}

pub trait SelfValidation: Sized {
    fn validate(&self) -> Result<()>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApplicationConfig {
    pub interval: String,
    pub checkers: Vec<CheckerConfig>,
    pub logger: LoggerConfig,
}

impl FromEnv for ApplicationConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            interval: DEFAULT_INTERVAL.to_owned(),
            checkers: Vec::new(),
            logger: LoggerConfig::from_env()?,
        })
    }
}

impl SelfValidation for ApplicationConfig {
    fn validate(&self) -> Result<()> {
        let _interval = duration_str::parse(self.interval.as_str())
            .with_context(|| format!("illegal interval: {}", self.interval.as_str()))?;
        for c in &self.checkers {
            let _ok = c.validate()?;
        }
        let _ok = self.logger.validate()?;
        Ok(())
    }
}

impl ApplicationConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let app_toml = ApplicationToml::load_from_file(path)?;
        let interval = if let Some(interval) = app_toml.interval {
            interval
        } else {
            DEFAULT_INTERVAL.to_owned()
        };
        let mut checkers = Vec::new();
        for checker in app_toml.checkers {
            let checker = checker.try_into()?;
            checkers.push(checker);
        }
        let logger = match app_toml.logger {
            Some(logger) => logger.try_into()?,
            None => LoggerConfig::from_env()?,
        };
        Ok(Self {
            interval,
            checkers,
            logger,
        })
    }

    pub fn get_interval(&self) -> Duration {
        duration_str::parse(self.interval.as_str()).expect("illegal interval config value...")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckerConfig {
    pub teritori_grpc_scheme: String,
    pub teritori_grpc_host: String,
    pub teritori_grpc_port: u16,
    pub validator_account: Option<String>,
    pub validator_address: Option<String>,
    pub syncing: bool,
    pub new_proposal: bool,
    pub missed_block: bool,
    pub missed_block_threshold: Option<MissedBlockThreshold>,
    pub validator_status: bool,
    pub slashes: bool,
}

impl CheckerConfig {
    pub fn teritori_grpc_endpoint(&self) -> String {
        format!(
            "{}://{}:{}",
            self.teritori_grpc_scheme, self.teritori_grpc_host, self.teritori_grpc_port
        )
    }
}

impl SelfValidation for CheckerConfig {
    fn validate(&self) -> Result<()> {
        if self.teritori_grpc_scheme != "http" && self.teritori_grpc_scheme != "https" {
            return Err(anyhow!(
                "teritori daemon grpc_scheme must be either 'http' or 'https'..."
            ));
        }
        let require_validator_account = self.missed_block;
        if require_validator_account {
            if self.validator_account.is_none() {
                return Err(anyhow!("validator_account is missing..."));
            }
        }
        let require_validator_address = self.validator_status || self.slashes;
        if require_validator_address {
            if self.validator_address.is_none() {
                return Err(anyhow!("validator_address is missing..."));
            }
        }
        Ok(())
    }
}

impl FromEnv for CheckerConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            teritori_grpc_scheme: "http".to_owned(),
            teritori_grpc_host: "127.0.0.1".to_owned(),
            teritori_grpc_port: 9090,
            validator_account: None,
            validator_address: None,
            syncing: true,
            new_proposal: false,
            missed_block: false,
            missed_block_threshold: None,
            validator_status: false,
            slashes: false,
        })
    }
}

impl TryFrom<CheckerToml> for CheckerConfig {
    type Error = anyhow::Error;

    fn try_from(toml: CheckerToml) -> Result<Self> {
        let mut result = Self::from_env()?;
        if let Some(teritori_grpc_scheme) = toml.teritori_grpc_scheme {
            result.teritori_grpc_scheme = teritori_grpc_scheme;
        }
        if let Some(teritori_grpc_host) = toml.teritori_grpc_host {
            result.teritori_grpc_host = teritori_grpc_host;
        }
        if let Some(teritori_grpc_port) = toml.teritori_grpc_port {
            result.teritori_grpc_port = teritori_grpc_port;
        }
        if let Some(validator_account) = toml.validator_account {
            result.validator_account = Some(validator_account);
        }
        if let Some(validator_address) = toml.validator_address {
            result.validator_address = Some(validator_address);
        }
        if let Some(syncing) = toml.syncing {
            result.syncing = syncing;
        }
        if let Some(new_proposal) = toml.new_proposal {
            result.new_proposal = new_proposal;
        }
        if let Some(missed_block) = toml.missed_block {
            result.missed_block = missed_block;
        }
        if let Some(missed_block_threshold) = toml.missed_block_threshold {
            result.missed_block_threshold = Some(missed_block_threshold.try_into()?);
        }
        if let Some(validator_status) = toml.validator_status {
            result.validator_status = validator_status;
        }
        if let Some(slashes) = toml.slashes {
            result.slashes = slashes;
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoggerConfig {
    pub chan_size: Option<usize>,
    pub is_async: bool,
    pub level: Option<String>,
    pub airbrake_host: Option<String>,
    pub airbrake_project_id: Option<String>,
    pub airbrake_project_key: Option<String>,
    pub airbrake_environment: Option<String>,
}

impl FromEnv for LoggerConfig {
    fn from_env() -> Result<Self> {
        Ok(Self {
            chan_size: None,
            is_async: true,
            level: None,
            airbrake_host: None,
            airbrake_project_id: None,
            airbrake_project_key: None,
            airbrake_environment: None,
        })
    }
}

impl SelfValidation for LoggerConfig {
    fn validate(&self) -> Result<()> {
        if let Some(level) = self.level.as_deref() {
            let valid_values = vec!["CRASH", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
            if valid_values.iter().find(|&&x| x == level).is_none() {
                return Err(anyhow!("illegal logger level: {}", level));
            }
        }
        Ok(())
    }
}

impl TryFrom<LoggerToml> for LoggerConfig {
    type Error = anyhow::Error;

    fn try_from(toml: LoggerToml) -> Result<Self> {
        let mut result = Self::from_env()?;
        if let Some(chan_size) = toml.chan_size {
            result.chan_size = Some(chan_size);
        }
        if let Some(is_async) = toml.is_async {
            result.is_async = is_async;
        }
        if let Some(level) = toml.level {
            result.level = Some(level);
        }
        if let Some(airbrake_host) = toml.airbrake_host {
            result.airbrake_host = Some(airbrake_host);
        }
        if let Some(airbrake_project_id) = toml.airbrake_project_id {
            result.airbrake_project_id = Some(airbrake_project_id);
        }
        if let Some(airbrake_project_key) = toml.airbrake_project_key {
            result.airbrake_project_key = Some(airbrake_project_key);
        }
        if let Some(airbrake_environment) = toml.airbrake_environment {
            result.airbrake_environment = Some(airbrake_environment);
        }
        Ok(result)
    }
}

#[allow(dead_code)]
fn get_env_var<T: FromStr>(var_name: &str, default_value: T) -> Result<T> {
    match env::var(var_name) {
        Ok(val) => val
            .parse::<T>()
            .map_err(|_| anyhow!("illegal env var: {}", var_name)),
        Err(_) => Ok(default_value),
    }
}
