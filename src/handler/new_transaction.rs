use crate::crypto::hash::Hashable;
use crate::miner::memory_pool::MemoryPool;
use crate::network::message::Message;
use crate::network::server::Handle;
use crate::transaction::Transaction;
use std::sync::Mutex;

use crate::config::*;
use std::sync::atomic::{AtomicU32, Ordering};

/// Handler for new transaction
// We may want to add the result of memory pool check
pub fn new_transaction(transaction: Transaction, mempool: &Mutex<MemoryPool>, server: &Handle) {
    let mut mempool = mempool.lock().unwrap();
    // memory pool check
    if !mempool.contains(&transaction.hash()) && !mempool.is_double_spend(&transaction.input) && (&transaction.shard_id == &SHARD_ID.load(Ordering::Relaxed) ) {
        // if check passes, insert the new transaction into the mempool
        //server.broadcast(Message::NewTransactionHashes(vec![transaction.hash()]));
        mempool.insert(transaction);
    }
    drop(mempool);
}


//RR_comment: Add transaction color filter here, transactions will have color
