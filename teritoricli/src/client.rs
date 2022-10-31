use crate::Result;
use anyhow::anyhow;
use anyhow::Context;
use logger::prelude::*;
use teritori_grpc_client as proto;
use teritori_grpc_client::tonic::{self, transport::Channel, Code};

#[derive(Debug, Clone)]
pub struct TeritoridClient {
    endpoint: String,
    connection: Option<Channel>,
}

impl TeritoridClient {
    pub async fn establish_connection(endpoint: &str) -> Result<Channel> {
        tonic::transport::Endpoint::new(endpoint.to_owned())?
            .connect()
            .await
            .with_context(|| {
                format!(
                    "failed to connect teritori daemon grpc endpoint: {}",
                    endpoint
                )
            })
    }

    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            connection: None,
        }
    }

    pub fn new_with_connection(endpoint: String, connection: Channel) -> Self {
        Self {
            endpoint,
            connection: Some(connection),
        }
    }

    async fn connect(&mut self) -> Result<()> {
        let connection = Self::establish_connection(self.endpoint.as_str()).await?;
        self.connection = Some(connection);
        Ok(())
    }

    async fn as_connection(&mut self) -> Result<Channel> {
        match self.connection.as_ref() {
            Some(conn) => Ok(conn.clone()),
            None => {
                self.connect().await?;
                Ok(self.connection.as_ref().unwrap().clone())
            }
        }
    }

    pub async fn fetch_syncing(&mut self) -> anyhow::Result<bool> {
        let mut client =
            proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::new(
                self.as_connection().await?,
            );
        let request =
            tonic::Request::new(proto::cosmos::base::tendermint::v1beta1::GetSyncingRequest {});
        let response = client.get_syncing(request).await.map_err(|status| {
            anyhow!(
                "unexpected response from {} status_code: {}, message: {}",
                self.endpoint,
                status.code(),
                status.message()
            )
        })?;
        Ok(response.into_inner().syncing)
    }

    pub async fn fetch_latest_block(
        &mut self,
    ) -> anyhow::Result<proto::cosmos::base::tendermint::v1beta1::GetLatestBlockResponse> {
        let mut client =
            proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::new(
                self.as_connection().await?,
            );
        let request =
            tonic::Request::new(proto::cosmos::base::tendermint::v1beta1::GetLatestBlockRequest {});
        let response = client.get_latest_block(request).await.map_err(|status| {
            anyhow!(
                "unexpected response from {} status_code: {}, message: {}",
                self.endpoint,
                status.code(),
                status.message()
            )
        })?;
        Ok(response.into_inner())
    }

    pub async fn fetch_block_by_height(
        &mut self,
        height: i64,
    ) -> anyhow::Result<proto::cosmos::base::tendermint::v1beta1::GetBlockByHeightResponse> {
        let mut client =
            proto::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::new(
                self.as_connection().await?,
            );
        let request = tonic::Request::new(
            proto::cosmos::base::tendermint::v1beta1::GetBlockByHeightRequest { height },
        );
        let response = client
            .get_block_by_height(request)
            .await
            .map_err(|status| {
                anyhow!(
                    "unexpected response from {} status_code: {}, message: {}",
                    self.endpoint,
                    status.code(),
                    status.message()
                )
            })?;
        Ok(response.into_inner())
    }

    pub async fn fetch_tx_by_hash(
        &mut self,
        tx_hash: &str,
    ) -> anyhow::Result<Option<proto::cosmos::base::abci::v1beta1::TxResponse>> {
        let mut client = proto::cosmos::tx::v1beta1::service_client::ServiceClient::new(
            self.as_connection().await?,
        );
        let request = tonic::Request::new(proto::cosmos::tx::v1beta1::GetTxRequest {
            hash: tx_hash.to_owned(),
        });
        let response = client.get_tx(request).await;
        match response {
            Ok(response) => Ok(response.into_inner().tx_response),
            Err(status) => match status.code() {
                Code::NotFound => Ok(None),
                Code::InvalidArgument => {
                    warn!(
                        "invalid argument response from {}, message: {}",
                        self.endpoint,
                        status.message()
                    );
                    Ok(None)
                }
                _ => Err(anyhow!(
                    "unexpected response from {} status_code: {}, message: {}",
                    self.endpoint,
                    status.code(),
                    status.message()
                )),
            },
        }
    }

    pub async fn fetch_validator_status(
        &mut self,
        validator_address: String,
    ) -> anyhow::Result<Option<proto::cosmos::staking::v1beta1::Validator>> {
        let mut client = proto::cosmos::staking::v1beta1::query_client::QueryClient::new(
            self.as_connection().await?,
        );
        let request = tonic::Request::new(proto::cosmos::staking::v1beta1::QueryValidatorRequest {
            validator_addr: validator_address,
        });
        let response = client.validator(request).await;
        match response {
            Ok(response) => Ok(response.into_inner().validator),
            Err(status) => match status.code() {
                Code::NotFound => Ok(None),
                Code::InvalidArgument => {
                    warn!(
                        "invalid argument response from {}, message: {}",
                        self.endpoint,
                        status.message()
                    );
                    Ok(None)
                }
                _ => Err(anyhow!(
                    "unexpected response from {} status_code: {}, message: {}",
                    self.endpoint,
                    status.code(),
                    status.message()
                )),
            },
        }
    }

    pub async fn fetch_slashes(
        &mut self,
        validator_address: String,
        starting_height: u64,
        ending_height: u64,
    ) -> anyhow::Result<Vec<proto::cosmos::distribution::v1beta1::ValidatorSlashEvent>> {
        let mut client = proto::cosmos::distribution::v1beta1::query_client::QueryClient::new(
            self.as_connection().await?,
        );
        let request = tonic::Request::new(
            proto::cosmos::distribution::v1beta1::QueryValidatorSlashesRequest {
                validator_address,
                starting_height,
                ending_height,
                pagination: None,
            },
        );
        let response = client.validator_slashes(request).await;
        match response {
            Ok(response) => Ok(response.into_inner().slashes),
            Err(status) => match status.code() {
                Code::NotFound => Ok(Vec::new()),
                Code::InvalidArgument => {
                    warn!(
                        "invalid argument response from {}, message: {}",
                        self.endpoint,
                        status.message()
                    );
                    Ok(Vec::new())
                }
                _ => Err(anyhow!(
                    "unexpected response from {} status_code: {}, message: {}",
                    self.endpoint,
                    status.code(),
                    status.message()
                )),
            },
        }
    }
}
