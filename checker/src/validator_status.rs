use channel::Receiver;
use futures::StreamExt;
use logger::prelude::*;
use std::sync::mpsc::SyncSender;

const BOND_STATUS_BONDED: i32 = 3;

#[derive(Debug)]
pub enum ValidatorStatusMessage {
    Check,
    Terminate(SyncSender<()>),
}

pub struct ValidatorStatusChecker {
    validator_address: String,
    teritorid_endpoint: String,
    receiver: Receiver<ValidatorStatusMessage>,
}

impl ValidatorStatusChecker {
    pub fn new(
        validator_address: String,
        teritorid_endpoint: String,
        receiver: Receiver<ValidatorStatusMessage>,
    ) -> Self {
        Self {
            validator_address,
            teritorid_endpoint,
            receiver,
        }
    }
    pub async fn run(mut self) {
        while let Some(message) = self.receiver.next().await {
            match message {
                ValidatorStatusMessage::Check => {
                    match teritoricli::get_client(&self.teritorid_endpoint)
                        .lock()
                        .await
                        .fetch_validator_status(self.validator_address.clone())
                        .await
                    {
                        Ok(validator) => {
                            if let Some(validator) = validator {
                                let mut has_error = false;
                                if validator.jailed {
                                    has_error = true;
                                }
                                if validator.status != BOND_STATUS_BONDED {
                                    has_error = true;
                                }
                                if has_error {
                                    error!(
                                        jailed = validator.jailed,
                                        status = validator.status,
                                        "validator {} is not healthy...",
                                        self.validator_address.as_str()
                                    )
                                } else {
                                    info!(
                                        "validator {} is healthy.",
                                        self.validator_address.as_str()
                                    );
                                }
                            } else {
                                warn!(
                                    "validator response is none for {}",
                                    self.validator_address.as_str()
                                );
                            }
                        }
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
                ValidatorStatusMessage::Terminate(sender) => {
                    info!("validator status checker will be terminated soon...");
                    let _ = sender.send(());
                    break;
                }
            }
        }
    }
}
