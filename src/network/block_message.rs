use crate::block::{Block,Header};
use crate::crypto::hash::H256;
use crate::transaction::Transaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockMessage {
    Block(Vec<u8>),
    BlockHeader(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlockData {
    Block(Block),
    BlockHeader(Header),
}
