use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::miner::memory_pool::MemoryPool;
use crate::network::message;
use crate::network::server::Handle as ServerHandle;
use crate::transaction::Transaction;
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use std::sync::Mutex;

use crate::config::*;
use std::sync::atomic::{AtomicU32, Ordering};

pub fn new_validated_block(
    block: &Block,
    mempool: &Mutex<MemoryPool>,
    blockdb: &BlockDatabase,
    chain: &BlockChain,
    server: &ServerHandle,
) {

    PERFORMANCE_COUNTER.record_process_block(&block);

    // if this block is a transaction, remove transactions from mempool
    match &block.content {
        Content::Transaction(content) => {
            if block.header.shard_id == SHARD_ID.load(Ordering::Relaxed){
                let mut mempool = mempool.lock().unwrap();
                for tx in &content.transactions {
                    mempool.remove_by_hash(&tx.hash());
                    // the inputs have been used here, so remove all transactions in the mempool that
                    // tries to use the input again.
                    for input in tx.input.iter() {
                        mempool.remove_by_input(input);
                    }
                }
                drop(mempool);
            }
        }
        _ => (),
    };

    // insert the new block into the blockchain
    chain.insert_block(&block).unwrap();
}

//RR_comment: Need to update it so that it only stores specific colored blocks
