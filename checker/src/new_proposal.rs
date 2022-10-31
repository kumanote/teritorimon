use crate::message::BlockMessage;
use crate::{utils, CustomError, Result};
use channel::Receiver;
use futures::StreamExt;
use logger::prelude::*;
use std::sync::mpsc::SyncSender;
use teritori_grpc_client::{self as proto, prost};

#[derive(Debug)]
pub enum NewProposalMessage {
    Check(BlockMessage),
    Terminate(SyncSender<()>),
}

impl From<BlockMessage> for NewProposalMessage {
    fn from(inner: BlockMessage) -> Self {
        Self::Check(inner)
    }
}

pub struct NewProposalChecker {
    teritorid_endpoint: String,
    receiver: Receiver<NewProposalMessage>,
}

impl NewProposalChecker {
    pub fn new(teritorid_endpoint: String, receiver: Receiver<NewProposalMessage>) -> Self {
        Self {
            teritorid_endpoint,
            receiver,
        }
    }
    pub async fn run(mut self) {
        while let Some(message) = self.receiver.next().await {
            match message {
                NewProposalMessage::Check(message) => {
                    if let Some(block) = message.block.as_ref() {
                        if let Some(data) = block.data.as_ref() {
                            for tx_bytes in &data.txs {
                                let tx: Result<proto::cosmos::tx::v1beta1::Tx> =
                                    prost::Message::decode(tx_bytes.as_slice()).map_err(|err| {
                                        CustomError::Transcode {
                                            reason: err.to_string(),
                                        }
                                        .into()
                                    });
                                let tx_hash = utils::calculate_hash(tx_bytes.as_slice())
                                    .expect("the tx bytes could not parse into hash...");
                                let mut tx_message_logs = None;
                                match tx {
                                    Ok(tx) => {
                                        if let Some(body) = tx.body.as_ref() {
                                            for (msg_index, tx_msg_any) in
                                                body.messages.iter().enumerate()
                                            {
                                                let type_url = tx_msg_any.type_url.as_str();
                                                if type_url
                                                    == "/cosmos.gov.v1beta1.MsgSubmitProposal"
                                                {
                                                    // check tx event here
                                                    let message_logs = if tx_message_logs.is_some()
                                                    {
                                                        tx_message_logs.as_ref().unwrap()
                                                    } else {
                                                        let tx_response = teritoricli::get_client(
                                                            &self.teritorid_endpoint,
                                                        )
                                                        .lock()
                                                        .await
                                                        .fetch_tx_by_hash(tx_hash.as_str())
                                                        .await;
                                                        if tx_response.is_err() {
                                                            error!("got error response while fetching tx detail: {}", tx_response.err().unwrap());
                                                            continue;
                                                        }
                                                        let tx_response = tx_response.unwrap();
                                                        if tx_response.is_none() {
                                                            warn!(
                                                                "tx is none: {}",
                                                                tx_hash.as_str()
                                                            );
                                                            continue;
                                                        }
                                                        let tx_response = tx_response.unwrap();
                                                        let message_logs_response =
                                                            tx_response.logs;
                                                        tx_message_logs =
                                                            Some(message_logs_response);
                                                        tx_message_logs.as_ref().unwrap()
                                                    };
                                                    let target_log =
                                                        message_logs.iter().find(|msg| {
                                                            msg.msg_index == msg_index as u32
                                                        });
                                                    let target_event = match target_log {
                                                        Some(log) => {
                                                            log.events.iter().find(|msg| {
                                                                msg.r#type == "submit_proposal"
                                                            })
                                                        }
                                                        None => None,
                                                    };
                                                    if let Some(target_event) = target_event {
                                                        let proposal_id = target_event
                                                            .attributes
                                                            .iter()
                                                            .find(|attr| attr.key == "proposal_id");
                                                        if let Some(proposal_id) = proposal_id {
                                                            error!(
                                                        "new proposal has just submitted. id: {}",
                                                        proposal_id.value.as_str()
                                                    )
                                                        };
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        warn!("transaction bytes could not parsed...{}", err);
                                    }
                                }
                            }
                        }
                    }
                }
                NewProposalMessage::Terminate(sender) => {
                    info!("new proposal checker will be terminated soon...");
                    let _ = sender.send(());
                    break;
                }
            }
        }
    }
}
