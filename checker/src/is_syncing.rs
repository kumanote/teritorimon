use channel::Receiver;
use futures::StreamExt;
use logger::prelude::*;
use std::sync::mpsc::SyncSender;

#[derive(Debug)]
pub enum IsSyncingMessage {
    Check,
    Terminate(SyncSender<()>),
}

pub struct IsSyncingChecker {
    teritorid_endpoint: String,
    receiver: Receiver<IsSyncingMessage>,
}

impl IsSyncingChecker {
    pub fn new(teritorid_endpoint: String, receiver: Receiver<IsSyncingMessage>) -> Self {
        Self {
            teritorid_endpoint,
            receiver,
        }
    }
    pub async fn run(mut self) {
        while let Some(message) = self.receiver.next().await {
            match message {
                IsSyncingMessage::Check => {
                    match teritoricli::get_client(&self.teritorid_endpoint)
                        .lock()
                        .await
                        .fetch_syncing()
                        .await
                    {
                        Ok(syncing) => {
                            if syncing {
                                error!(
                                    "the teritori daemon: {} is syncing",
                                    self.teritorid_endpoint.as_str()
                                );
                            } else {
                                info!(
                                    "the teritori daemon: {} is synced",
                                    self.teritorid_endpoint.as_str()
                                );
                            }
                        }
                        Err(err) => {
                            error!("{}", err);
                        }
                    }
                }
                IsSyncingMessage::Terminate(sender) => {
                    info!("is syncing checker will be terminated soon...");
                    let _ = sender.send(());
                    break;
                }
            }
        }
    }
}
