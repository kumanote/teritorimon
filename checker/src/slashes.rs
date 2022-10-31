use channel::Receiver;
use futures::StreamExt;
use logger::prelude::*;
use std::sync::mpsc::SyncSender;

#[derive(Debug)]
pub enum SlashesMessage {
    Check(SlashesMessageParams),
    Terminate(SyncSender<()>),
}

#[derive(Debug)]
pub struct SlashesMessageParams {
    pub starting_height: u64,
    pub ending_height: u64,
}

pub struct SlashesChecker {
    validator_address: String,
    teritorid_endpoint: String,
    receiver: Receiver<SlashesMessage>,
}

impl SlashesChecker {
    pub fn new(
        validator_address: String,
        teritorid_endpoint: String,
        receiver: Receiver<SlashesMessage>,
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
                SlashesMessage::Check(params) => {
                    match teritoricli::get_client(&self.teritorid_endpoint)
                        .lock()
                        .await
                        .fetch_slashes(
                            self.validator_address.clone(),
                            params.starting_height,
                            params.ending_height,
                        )
                        .await
                    {
                        Ok(slash_events) => {
                            if slash_events.len() > 0 {
                                error!(
                                    "validator {} has slash event between {} and {}",
                                    self.validator_address.as_str(),
                                    params.starting_height,
                                    params.ending_height,
                                );
                            } else {
                                info!(
                                    "validator {} has no slash event between {} and {}",
                                    self.validator_address.as_str(),
                                    params.starting_height,
                                    params.ending_height,
                                );
                            }
                        }
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
                SlashesMessage::Terminate(sender) => {
                    info!("slashes checker will be terminated soon...");
                    let _ = sender.send(());
                    break;
                }
            }
        }
    }
}
