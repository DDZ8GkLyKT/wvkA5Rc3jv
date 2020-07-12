use crate::block::Block;
use crate::crypto::hash::H256;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    NewBlockHashes(Vec<(u32,u32,H256)>),
    NewTransactionHashes(Vec<(u32,H256)>),
    GetBlocks(Vec<(u32,u32,H256)>),
    Blocks(Vec<Vec<u8>>),
    GetTransactions(Vec<H256>),
    Transactions(Vec<Transaction>),
    Bootstrap(H256),
}

/*
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHashWShardID{
    shard_id: u32,
    block_hash: H256,
}
*/
