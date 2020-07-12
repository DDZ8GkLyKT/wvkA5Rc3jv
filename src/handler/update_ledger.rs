use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::Hashable;
use crate::miner::memory_pool::MemoryPool;
use crate::network::message;
use crate::network::server::Handle as ServerHandle;
use crate::transaction::Transaction;
use crate::utxodb::UtxoDatabase;
use crate::wallet::Wallet;
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use std::sync::Mutex;

use crate::config::*;
use std::sync::atomic::{AtomicU64, Ordering};


pub fn update_ledger(
    blockdb: &BlockDatabase,
    chain: &BlockChain,
    utxodb: &UtxoDatabase,
    wallet: &Wallet,
) {
    let diff = chain.update_ledger().unwrap();
    PERFORMANCE_COUNTER.record_confirm_transaction_blocks(diff.0.len());
    PERFORMANCE_COUNTER.record_deconfirm_transaction_blocks(diff.1.len());

    // gather the transaction diff and apply on utxo database
    let mut add: Vec<Transaction> = vec![];
    let mut remove: Vec<Transaction> = vec![];
    //println!("my_sid: {}", SHARD_ID.load(Ordering::Relaxed));
    for (block_sid,hash) in diff.0 {
        println!("Adding blocks with block_sid: {}, hash: {}", block_sid, hash);
        if block_sid == SHARD_ID.load(Ordering::Relaxed){
            let block = blockdb.get(&hash).unwrap().unwrap();
            let content = match block.content {
                Content::Transaction(data) => data,
                _ => unreachable!(),
            };
            let mut transactions = content.transactions.clone();
            add.append(&mut transactions);
            println!("adding transactions");
        }

    }
    for (block_sid,hash) in diff.1 {
        if block_sid == SHARD_ID.load(Ordering::Relaxed){
            let block = blockdb.get(&hash).unwrap().unwrap();
            let content = match block.content {
                Content::Transaction(data) => data,
                _ => unreachable!(),
            };
            let mut transactions = content.transactions.clone();
            remove.append(&mut transactions);
            //println!("removing transactions");
        }
    }

    //println!("Calling utxodb.apply_diff");

    let coin_diff = utxodb.apply_diff(&add, &remove).unwrap();
    wallet.apply_diff(&coin_diff.0, &coin_diff.1).unwrap();
}
