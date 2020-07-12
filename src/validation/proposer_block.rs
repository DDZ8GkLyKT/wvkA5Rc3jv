use super::{check_proposer_block_exists, check_transaction_block_exists};
use crate::block::proposer::Content;
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::H256;

use crate::config::*;
use std::sync::atomic::{AtomicU32, Ordering};

pub fn get_missing_references(
    content: &Content,
    blockchain: &BlockChain,
    blockdb: &BlockDatabase,
) -> Vec<(u32,u32,H256)> {
    let mut missing_blocks: Vec<(u32,u32,H256)> = vec![];

    // check whether the tx block referred are present
    for (block_sid,tx_block_hash) in content.transaction_refs.iter() {  //RR_future: Check only txblocks of same shard for validity check
        //if block_sid == &SHARD_ID.load(Ordering::Relaxed){
        let tx_block = check_transaction_block_exists(*tx_block_hash, blockchain);
        if !tx_block {
            missing_blocks.push((2,*block_sid,*tx_block_hash));
        }
        //}

    }

    // check whether the proposer blocks referred are present
    for prop_block_hash in content.proposer_refs.iter() {
        let prop_block = check_proposer_block_exists(*prop_block_hash, blockchain);
        if !prop_block {
            missing_blocks.push((0,0,*prop_block_hash)); //Setting shard id as 0 since it does not matter
        }
    }

    return missing_blocks;
}

pub fn check_ref_proposer_level(parent: &H256, content: &Content, blockchain: &BlockChain) -> bool {
    let parent_level = match blockchain.proposer_level(parent) {
        Ok(l) => l,
        _ => return false,
    };
    for prop_block_hash in content.proposer_refs.iter() {
        match blockchain.proposer_level(prop_block_hash) {
            Ok(l) => {
                if l > parent_level {
                    return false;
                }
            }
            _ => return false,
        }
    }
    return true;
}
