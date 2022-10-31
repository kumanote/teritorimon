use teritori_grpc_client as proto;

#[derive(Debug, Clone)]
pub struct BlockMessage {
    pub block_id: Option<proto::tendermint::types::BlockId>,
    pub block: Option<proto::tendermint::types::Block>,
}

impl From<proto::cosmos::base::tendermint::v1beta1::GetLatestBlockResponse> for BlockMessage {
    fn from(msg: proto::cosmos::base::tendermint::v1beta1::GetLatestBlockResponse) -> Self {
        Self {
            block_id: msg.block_id,
            block: msg.block,
        }
    }
}

impl From<proto::cosmos::base::tendermint::v1beta1::GetBlockByHeightResponse> for BlockMessage {
    fn from(msg: proto::cosmos::base::tendermint::v1beta1::GetBlockByHeightResponse) -> Self {
        Self {
            block_id: msg.block_id,
            block: msg.block,
        }
    }
}
