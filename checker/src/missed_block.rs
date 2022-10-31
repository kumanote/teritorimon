use crate::message::BlockMessage;
use channel::Receiver;
use config::MissedBlockThreshold;
use crypto::account;
use futures::StreamExt;
use logger::prelude::*;
use std::sync::mpsc::SyncSender;

#[derive(Debug)]
pub enum MissedBlockMessage {
    Check(BlockMessage),
    Terminate(SyncSender<()>),
}

impl From<BlockMessage> for MissedBlockMessage {
    fn from(inner: BlockMessage) -> Self {
        Self::Check(inner)
    }
}

pub struct MissedBlockChecker {
    validator_account: account::Id,
    missed_block_threshold: MissedBlockThreshold,
    receiver: Receiver<MissedBlockMessage>,
}

impl MissedBlockChecker {
    pub fn new(
        validator_account: account::Id,
        missed_block_threshold: MissedBlockThreshold,
        receiver: Receiver<MissedBlockMessage>,
    ) -> Self {
        Self {
            validator_account,
            missed_block_threshold,
            receiver,
        }
    }

    pub async fn run(mut self) {
        let mut missed_block_heights = vec![];
        let missed_block_threshold = self.missed_block_threshold;
        let validator_address = self.validator_account.clone().to_string();
        let validator_address_bytes = self.validator_account.as_bytes();
        while let Some(message) = self.receiver.next().await {
            match message {
                MissedBlockMessage::Check(message) => {
                    if let Some(block) = message.block.as_ref() {
                        if let Some(commit) = block.last_commit.as_ref() {
                            let block_height = block.header.as_ref().unwrap().height;
                            let signed = commit
                                .signatures
                                .iter()
                                .find(|s| {
                                    trace!(
                                        "signature: {} detected!",
                                        account::Id::try_from(s.validator_address.clone()).unwrap()
                                    );
                                    s.validator_address.as_slice() == validator_address_bytes
                                })
                                .is_some();

                            let lowest =
                                block_height - (missed_block_threshold.denominator as i64) + 1;
                            missed_block_heights = missed_block_heights
                                .clone()
                                .into_iter()
                                .filter(|&h| h >= lowest)
                                .collect();

                            if signed {
                                info!(
                                    "{} has signed for block {}",
                                    validator_address.as_str(),
                                    block_height
                                )
                            } else {
                                missed_block_heights.push(block_height);
                                if missed_block_heights.len()
                                    >= missed_block_threshold.numerator as usize
                                {
                                    error!(
                                        "{} has not signed for block {}",
                                        validator_address.as_str(),
                                        block_height
                                    )
                                } else {
                                    warn!(
                                        "{} has not signed for block {} but under threshold",
                                        validator_address.as_str(),
                                        block_height
                                    )
                                }
                            }
                        }
                    }
                }
                MissedBlockMessage::Terminate(sender) => {
                    info!("missed block checker will be terminated soon...");
                    let _ = sender.send(());
                    break;
                }
            }
        }
    }
}
