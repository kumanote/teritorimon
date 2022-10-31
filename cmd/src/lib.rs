use anyhow::anyhow;
use channel::Sender;
use checker;
use checker::is_syncing::IsSyncingMessage;
use checker::message::BlockMessage;
use checker::missed_block::MissedBlockMessage;
use checker::new_proposal::NewProposalMessage;
use checker::slashes::{SlashesMessage, SlashesMessageParams};
use checker::validator_status::ValidatorStatusMessage;
use config::MissedBlockThreshold;
use crypto::account;
use logger::prelude::*;
use std::str::FromStr;
use std::thread;
use tokio::signal::unix::{signal, SignalKind};

pub type Result<T> = anyhow::Result<T>;

pub fn start() -> Result<()> {
    let app_config = config::app_config();

    let mut managers = Vec::new();
    let mut runtimes = Vec::new();
    for checker in &app_config.checkers {
        let mut manager = CheckManager::default();
        if let Some(validator_account) = checker.validator_account.as_deref() {
            manager.validator_account(validator_account);
        }
        if let Some(validator_address) = checker.validator_address.as_deref() {
            manager.validator_address(validator_address);
        }
        let runtime = manager
            .teritorid_endpoint(checker.teritori_grpc_endpoint().as_str())
            .check_if_syncing(checker.syncing)
            .check_if_new_proposal(checker.new_proposal)
            .check_if_missed_block(checker.missed_block, checker.missed_block_threshold)
            .check_if_validator_status(checker.validator_status)
            .check_if_slashes(checker.slashes)
            .setup();
        managers.push(manager);
        runtimes.push(runtime);
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("tick")
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime!");
    let interval = app_config.get_interval();

    runtime.block_on(async move {
        let mut sigint =
            signal(SignalKind::interrupt()).expect("signal interrupt must be captured...");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("signal termination must be captured...");
        let mut sleep = false;
        loop {
            let tick = async {
                if sleep {
                    thread::sleep(interval);
                    debug!("next tick");
                }
            };
            tokio::select! {
                _ = sigint.recv() => {
                    info!("sigint detected");
                    for manager in &mut managers {
                        manager.terminate();
                    }
                    break
                }
                _ = sigterm.recv() => {
                    info!("sigterm detected");
                    for manager in &mut managers {
                        manager.terminate();
                    }
                    break
                }
                _ = tick => {
                    for manager in &mut managers {
                        if let Err(err) = manager.next().await {
                            error!("{}", err);
                        }
                    }
                    sleep = true;
                }
            }
        }
    });
    Ok(())
}

pub struct CheckManager {
    teritorid_endpoint: String,
    validator_account: Option<account::Id>,
    validator_address: Option<String>,
    check_if_syncing: bool,
    is_syncing_checker: Option<Sender<IsSyncingMessage>>,
    check_if_new_proposal: bool,
    new_proposal_checker: Option<Sender<NewProposalMessage>>,
    check_if_missed_block: bool,
    missed_block_threshold: Option<MissedBlockThreshold>,
    missed_block_checker: Option<Sender<MissedBlockMessage>>,
    check_if_validator_status: bool,
    validator_status_checker: Option<Sender<ValidatorStatusMessage>>,
    check_if_slashes: bool,
    slashes_checker: Option<Sender<SlashesMessage>>,
    latest_height: Option<i64>,
}

impl Default for CheckManager {
    fn default() -> Self {
        Self {
            teritorid_endpoint: "".to_owned(),
            validator_account: None,
            validator_address: None,
            check_if_syncing: true,
            is_syncing_checker: None,
            check_if_new_proposal: true,
            new_proposal_checker: None,
            check_if_missed_block: false,
            missed_block_threshold: None,
            missed_block_checker: None,
            check_if_validator_status: false,
            validator_status_checker: None,
            check_if_slashes: false,
            slashes_checker: None,
            latest_height: None,
        }
    }
}

impl CheckManager {
    pub fn teritorid_endpoint(&mut self, teritorid_endpoint: &str) -> &mut Self {
        self.teritorid_endpoint = teritorid_endpoint.to_owned();
        self
    }
    pub fn validator_account(&mut self, validator_account: &str) -> &mut Self {
        let validator_account = account::Id::from_str(validator_account)
            .expect("validator account must be in valid hex string.");
        self.validator_account = Some(validator_account);
        self
    }
    pub fn validator_address(&mut self, validator_address: &str) -> &mut Self {
        self.validator_address = Some(validator_address.to_owned());
        self
    }
    pub fn check_if_syncing(&mut self, check_if_syncing: bool) -> &mut Self {
        self.check_if_syncing = check_if_syncing;
        self
    }
    pub fn check_if_new_proposal(&mut self, check_if_new_proposal: bool) -> &mut Self {
        self.check_if_new_proposal = check_if_new_proposal;
        self
    }
    pub fn check_if_missed_block(
        &mut self,
        check_if_missed_block: bool,
        missed_block_threshold: Option<MissedBlockThreshold>,
    ) -> &mut Self {
        self.check_if_missed_block = check_if_missed_block;
        self.missed_block_threshold = missed_block_threshold;
        self
    }
    pub fn check_if_validator_status(&mut self, check_if_validator_status: bool) -> &mut Self {
        self.check_if_validator_status = check_if_validator_status;
        self
    }
    pub fn check_if_slashes(&mut self, check_if_slashes: bool) -> &mut Self {
        self.check_if_slashes = check_if_slashes;
        self
    }
    fn setup(&mut self) -> tokio::runtime::Runtime {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name(format!("{}", self.teritorid_endpoint.as_str()))
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime!");
        if self.check_if_syncing {
            let (sender, receiver) = channel::new(1_024);
            let checker = checker::is_syncing::IsSyncingChecker::new(
                self.teritorid_endpoint.clone(),
                receiver,
            );
            runtime.spawn(checker.run());
            self.is_syncing_checker = Some(sender);
        }
        if self.check_if_new_proposal {
            let (sender, receiver) = channel::new(1_024);
            let checker = checker::new_proposal::NewProposalChecker::new(
                self.teritorid_endpoint.clone(),
                receiver,
            );
            runtime.spawn(checker.run());
            self.new_proposal_checker = Some(sender);
        }
        if self.check_if_missed_block {
            let (sender, receiver) = channel::new(1_024);
            let validator_account = self
                .validator_account
                .as_ref()
                .expect("validator account must be provided to check missed blocks.")
                .clone();
            let threshold = self
                .missed_block_threshold
                .unwrap_or(MissedBlockThreshold::default());
            let checker = checker::missed_block::MissedBlockChecker::new(
                validator_account,
                threshold,
                receiver,
            );
            runtime.spawn(checker.run());
            self.missed_block_checker = Some(sender);
        }
        if self.check_if_validator_status {
            let (sender, receiver) = channel::new(1_024);
            let validator_address = self
                .validator_address
                .as_ref()
                .expect("validator address must be provided to check validator status.")
                .clone();
            let checker = checker::validator_status::ValidatorStatusChecker::new(
                validator_address,
                self.teritorid_endpoint.clone(),
                receiver,
            );
            runtime.spawn(checker.run());
            self.validator_status_checker = Some(sender);
        }
        if self.check_if_slashes {
            let (sender, receiver) = channel::new(1_024);
            let validator_address = self
                .validator_address
                .as_ref()
                .expect("validator address must be provided to check validator status.")
                .clone();
            let checker = checker::slashes::SlashesChecker::new(
                validator_address,
                self.teritorid_endpoint.clone(),
                receiver,
            );
            runtime.spawn(checker.run());
            self.slashes_checker = Some(sender);
        }
        runtime
    }

    fn terminate(&mut self) {
        if let Some(sender) = self.is_syncing_checker.as_mut() {
            let (oneshot_sender, oneshot_receiver) = std::sync::mpsc::sync_channel(1);
            sender
                .try_send(IsSyncingMessage::Terminate(oneshot_sender))
                .unwrap();
            oneshot_receiver.recv().unwrap();
        }
        if let Some(sender) = self.new_proposal_checker.as_mut() {
            let (oneshot_sender, oneshot_receiver) = std::sync::mpsc::sync_channel(1);
            sender
                .try_send(NewProposalMessage::Terminate(oneshot_sender))
                .unwrap();
            oneshot_receiver.recv().unwrap();
        }
        if let Some(sender) = self.missed_block_checker.as_mut() {
            let (oneshot_sender, oneshot_receiver) = std::sync::mpsc::sync_channel(1);
            sender
                .try_send(MissedBlockMessage::Terminate(oneshot_sender))
                .unwrap();
            oneshot_receiver.recv().unwrap();
        }
        if let Some(sender) = self.validator_status_checker.as_mut() {
            let (oneshot_sender, oneshot_receiver) = std::sync::mpsc::sync_channel(1);
            sender
                .try_send(ValidatorStatusMessage::Terminate(oneshot_sender))
                .unwrap();
            oneshot_receiver.recv().unwrap();
        }
        if let Some(sender) = self.slashes_checker.as_mut() {
            let (oneshot_sender, oneshot_receiver) = std::sync::mpsc::sync_channel(1);
            sender
                .try_send(SlashesMessage::Terminate(oneshot_sender))
                .unwrap();
            oneshot_receiver.recv().unwrap();
        }
    }

    async fn next(&mut self) -> Result<&mut Self> {
        if let Some(sender) = self.is_syncing_checker.as_mut() {
            sender
                .try_send(IsSyncingMessage::Check)
                .map_err(|err| anyhow!("{}", err))?;
        }
        if let Some(sender) = self.validator_status_checker.as_mut() {
            sender
                .try_send(ValidatorStatusMessage::Check)
                .map_err(|err| anyhow!("{}", err))?;
        }
        let latest_block_response = teritoricli::get_client(&self.teritorid_endpoint)
            .lock()
            .await
            .fetch_latest_block()
            .await?;
        let latest_height = latest_block_response
            .block
            .as_ref()
            .unwrap()
            .header
            .as_ref()
            .unwrap()
            .height;
        let from_height = match self.latest_height {
            Some(last_checked_height) => last_checked_height + 1,
            None => latest_height,
        };
        if from_height <= latest_height {
            for height in from_height..=latest_height {
                let block_message: BlockMessage = if height == latest_height {
                    latest_block_response.clone().into()
                } else {
                    let block_response = teritoricli::get_client(&self.teritorid_endpoint)
                        .lock()
                        .await
                        .fetch_block_by_height(height)
                        .await?;
                    block_response.into()
                };
                if let Some(sender) = self.new_proposal_checker.as_mut() {
                    info!(
                        "let's check if new proposal was submitted inside {} height block!",
                        height
                    );
                    sender
                        .try_send(block_message.clone().into())
                        .map_err(|err| anyhow!("{}", err))?;
                }
                if let Some(sender) = self.missed_block_checker.as_mut() {
                    info!(
                        "let's check if validator: {} has missed to sign {} height block,",
                        self.validator_address.as_deref().unwrap(),
                        height
                    );
                    sender
                        .try_send(block_message.clone().into())
                        .map_err(|err| anyhow!("{}", err))?;
                }
            }
            if let Some(sender) = self.slashes_checker.as_mut() {
                sender
                    .try_send(SlashesMessage::Check(SlashesMessageParams {
                        starting_height: from_height as u64,
                        ending_height: latest_height as u64,
                    }))
                    .map_err(|err| anyhow!("{}", err))?;
            }
            self.latest_height = Some(latest_height)
        }
        Ok(self)
    }
}
