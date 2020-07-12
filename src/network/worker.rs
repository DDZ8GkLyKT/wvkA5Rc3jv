use super::buffer::BlockBuffer;
use super::message::{self, Message};
use super::peer;
use crate::block::{Block, Content};
use crate::blockchain::BlockChain;
use crate::blockdb::BlockDatabase;
use crate::crypto::hash::{Hashable, H256};
use crate::experiment::performance_counter::PERFORMANCE_COUNTER;
use crate::handler::new_transaction;
use crate::handler::new_validated_block;
use crate::miner::memory_pool::MemoryPool;
use crate::miner::ContextUpdateSignal;
use crate::network::server::Handle as ServerHandle;
use crate::utxodb::UtxoDatabase;
use crate::validation::{self, BlockResult};
use crate::wallet::Wallet;
use crossbeam::channel;
use log::{debug, info, warn};

use std::collections::{HashSet,HashMap};
use std::sync::{mpsc, Arc, Mutex};

use std::collections::VecDeque;
use std::iter::FromIterator;

use std::thread;

use crate::config::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone)]
pub struct Context {
    msg_chan: channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    chain: Arc<BlockChain>,
    blockdb: Arc<BlockDatabase>,
    utxodb: Arc<UtxoDatabase>,
    wallet: Arc<Wallet>,
    mempool: Arc<Mutex<MemoryPool>>,
    context_update_chan: channel::Sender<ContextUpdateSignal>,
    server: ServerHandle,
    buffer: Arc<Mutex<BlockBuffer>>,
    processed_foreign_blocks: Arc<Mutex<HashSet<H256>>>,
    recent_blocks: Arc<Mutex<HashSet<H256>>>,   // blocks that we have received but not yet inserted
    requested_blocks: Arc<Mutex<HashSet<H256>>>,    // blocks that we have requested but not yet received
}

pub fn new(
    num_worker: usize,
    msg_src: channel::Receiver<(Vec<u8>, peer::Handle)>,
    blockchain: &Arc<BlockChain>,
    blockdb: &Arc<BlockDatabase>,
    utxodb: &Arc<UtxoDatabase>,
    wallet: &Arc<Wallet>,
    mempool: &Arc<Mutex<MemoryPool>>,
    ctx_update_sink: channel::Sender<ContextUpdateSignal>,
    server: &ServerHandle,
) -> Context {
    let ctx = Context {
        msg_chan: msg_src,
        num_worker: num_worker,
        chain: Arc::clone(blockchain),
        blockdb: Arc::clone(blockdb),
        utxodb: Arc::clone(utxodb),
        wallet: Arc::clone(wallet),
        mempool: Arc::clone(mempool),
        context_update_chan: ctx_update_sink,
        server: server.clone(),
        buffer: Arc::new(Mutex::new(BlockBuffer::new())),
        processed_foreign_blocks: Arc::new(Mutex::new(HashSet::new())),
        recent_blocks: Arc::new(Mutex::new(HashSet::new())),
        requested_blocks: Arc::new(Mutex::new(HashSet::new())),
    };
    return ctx;
}

impl Context {
    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let msg = self.msg_chan.recv().unwrap();
            PERFORMANCE_COUNTER.record_process_message();
            let (msg, peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewTransactionHashes(hashes) => {
                    debug!("Got {} new transaction hashes", hashes.len());
                    let mut hashes_to_request = vec![];
                    for hash in hashes {
                        if hash.0==SHARD_ID.load(Ordering::Relaxed){
                            if !self.mempool.lock().unwrap().contains(&hash.1) {
                                hashes_to_request.push(hash.1);
                            }
                        }

                    }
                    if hashes_to_request.len() != 0 {
                        peer.write(Message::GetTransactions(hashes_to_request));
                    }
                }
                Message::GetTransactions(hashes) => {
                    debug!("Asked for {} transactions", hashes.len());
                    let mut transactions = vec![];
                    for hash in hashes {
                        match self.mempool.lock().unwrap().get(&hash) {
                            None => {}
                            Some(entry) => {
                                transactions.push(entry.transaction.clone());
                            }
                        }
                    }
                    peer.write(Message::Transactions(transactions));
                }
                Message::Transactions(transactions) => {
                    debug!("Got {} transactions", transactions.len());
                    for transaction in transactions {
                        new_transaction(transaction, &self.mempool, &self.server);
                    }
                }
                //RR_comment: Update the newblock and new transaction logic to ignore blocks and transactions in other shards
                Message::NewBlockHashes(hashes) => {
                    debug!("Got {} new block hashes", hashes.len());
                    let mut hashes_to_request = vec![];
                    for (content_id, shard_id, hash) in hashes {        //RR_future: add shard id (hash.0) filtering after data availability issue is resolved
                        // we need to check blockchain as well
                        let requested_blocks = self.requested_blocks.lock().unwrap();
                        let requested = requested_blocks.contains(&hash);
                        drop(requested_blocks);
                        if !(shard_id != SHARD_ID.load(Ordering::Relaxed) && content_id==2){
                            let in_blockdb = self.blockdb.contains(&hash).unwrap();
                            if !(in_blockdb || requested) {
                                hashes_to_request.push((content_id, shard_id, hash));
                            }
                        } else {
                            let mut processed_foreign_blocks = self.processed_foreign_blocks.lock().unwrap();
                            let in_foreign_hashmap = processed_foreign_blocks.contains(&hash);
                            if !(in_foreign_hashmap || requested){
                                drop(processed_foreign_blocks);
                                hashes_to_request.push((content_id, shard_id, hash));
                            }
                        }
                    }
                    let mut requested_blocks = self.requested_blocks.lock().unwrap();
                    for (content_id, shard_id, hash) in &hashes_to_request {
                        requested_blocks.insert(*hash);
                    }
                    drop(requested_blocks);
                    if hashes_to_request.len() != 0 {
                        peer.write(Message::GetBlocks(hashes_to_request));
                    }
                }
                Message::GetBlocks(hashes) => {
                    debug!("Asked for {} blocks", hashes.len());
                    let mut blocks = vec![];
                    let my_shard_id: u32 = SHARD_ID.load(Ordering::Relaxed);
                    for (content_id, shard_id, hash) in hashes {
                        if !(shard_id != my_shard_id && content_id==2){
                            match self.blockdb.get_encoded(&hash).unwrap() {
                                None => {}
                                Some(encoded_block) => {
                                    blocks.push(encoded_block.to_vec());
                                }
                            }
                        }
                        else {
                            match self.blockdb.get_encoded_foreign(&hash, shard_id).unwrap() {
                                None => {}
                                Some(encoded_block) => {
                                    blocks.push(encoded_block.to_vec());
                                }
                            }
                        }

                    }
                    peer.write(Message::Blocks(blocks));
                }
                Message::Blocks(encoded_blocks) => {
                    debug!("Got {} blocks", encoded_blocks.len());

                    // decode the blocks
                    let mut blocks: Vec<Block> = vec![];
                    let mut hashes: Vec<(u32, u32, H256)> = vec![];
                    let mut hashes_to_broadcast: Vec<(u32, u32, H256)> = vec![];
                    let my_shard_id: u32 = SHARD_ID.load(Ordering::Relaxed);

                    for encoded_block in &encoded_blocks {
                        let block: Block = bincode::deserialize(&encoded_block).unwrap();
                        let block_shard_id = block.header.shard_id;
                        let hash = block.hash();
                        let mut block_content_id: u32=0;
                        match &block.content {
                            Content::Transaction(_) => {
                                block_content_id =2;
                            }
                            Content::Voter(_) => {
                                block_content_id=1;
                            }
                            Content::Proposer(_) => {
                                block_content_id=0;
                            }
                        }
                        // now that the block that we request has arrived, remove it from the set
                        // of requested blocks. removing it at this stage causes a race condition,
                        // where the block could have been removed from requested_blocks but not
                        // yet inserted into the database. but this does not cause correctness
                        // problem and hardly incurs a performance issue (I hope)
                        let mut requested_blocks = self.requested_blocks.lock().unwrap();
                        requested_blocks.remove(&hash);
                        drop(requested_blocks);
                        // check POW here. If POW does not pass, discard the block at this
                        // stage
                        let pow_check = validation::check_pow_sortition_id(&block);
                        match pow_check {
                            BlockResult::Pass => {}
                            _ => continue,
                        }

                        // check whether the block is being processed. note that here we use lock
                        // to make sure that the hash either in recent_blocks, or blockdb, so we
                        // don't have a single duplicate
                        let mut recent_blocks = self.recent_blocks.lock().unwrap();
                        if recent_blocks.contains(&hash) {
                            drop(recent_blocks);
                            continue;
                        }
                        // register this block as being processed
                        recent_blocks.insert(hash);
                        drop(recent_blocks);

                        // TODO: consider the ordering here. I'd expect a lot of duplicate blocks
                        // to proceed to this step, which means a lot of useless database lookups
                        // and lock/unlocks
                        // detect duplicates
                        if self.blockdb.contains(&hash).unwrap() {
                            let mut recent_blocks = self.recent_blocks.lock().unwrap();
                            recent_blocks.remove(&hash);
                            drop(recent_blocks);
                            continue;
                        }

                        let mut processed_foreign_blocks = self.processed_foreign_blocks.lock().unwrap();
                        if processed_foreign_blocks.contains(&hash){
                            drop(processed_foreign_blocks);
                            let mut recent_blocks = self.recent_blocks.lock().unwrap();
                            recent_blocks.remove(&hash);
                            continue;
                        } else {
                            drop(processed_foreign_blocks);
                        }

                        // store the block into database
                        if !(block_shard_id != my_shard_id && block_content_id==2){
                            self.blockdb.insert_encoded(&hash, &encoded_block).unwrap();// RR_future: Insert the block regardless of shard id, done in order to resolve the DA issue
                        } else {
                            let mut processed_foreign_blocks = self.processed_foreign_blocks.lock().unwrap();
                            processed_foreign_blocks.insert(hash);
                            drop(processed_foreign_blocks);
                            self.blockdb.insert_encoded_foreign(&hash, &encoded_block, block_shard_id).unwrap();
                        }


                        //self.blockdb.insert_encoded(&hash, &encoded_block).unwrap();// RR_future: Insert the block regardless of shard id, done in order to resolve the DA issue
                        //RR_future: Need to delete the transaction blocks of other shards after some confirmation threshold
                        // Do not run it if transaction blocks of different shards are received
                        // insert the new block into the blockchain
                        //self.chain.insert_block(&block).unwrap(); //RR_caution: done to insert transaction blockhashes in the chain for use of miner

                        // now that this block is store, remove the reference
                        let mut recent_blocks = self.recent_blocks.lock().unwrap();
                        recent_blocks.remove(&hash);
                        drop(recent_blocks);

                        blocks.push(block);
                        hashes.push((block_content_id, block_shard_id,  hash));
                        if !(block_shard_id != my_shard_id && block_content_id==2){
                            hashes_to_broadcast.push((block_content_id, block_shard_id,  hash));
                        }
                        else {
                            hashes_to_broadcast.push((block_content_id, block_shard_id,  hash));
                        }


                    }

                    for block in &blocks {
                        PERFORMANCE_COUNTER.record_receive_block(&block);
                    }

                    // tell peers about the new blocks
                    // TODO: we will do this only in a reasonable network topology
                    if hashes.is_empty() {
                        continue; // end processing this message
                    }
                    self.server
                        .broadcast(Message::NewBlockHashes(hashes_to_broadcast.clone()));

                    // process each block
                    let mut to_process: Vec<Block> = blocks;
                    let mut to_request: Vec<(u32,u32,H256)> = vec![];
                    let mut context_update_sig = vec![];
                    while let Some(block) = to_process.pop() {

                        // check data availability
                        // make sure checking data availability and buffering are one atomic
                        // operation. see the comments in buffer.rs
                        let mut buffer = self.buffer.lock().unwrap();
                        let data_availability =
                            validation::check_data_availability(&block, &self.chain, &self.blockdb);
                        match data_availability {
                            BlockResult::Pass => drop(buffer),
                            BlockResult::MissingReferences(r) => {
                                debug!(
                                    "Missing {} referred blocks for block {:.8}",
                                    r.len(),
                                    block.hash()
                                );
                                buffer.insert(block, &r);
                                to_request.extend_from_slice(&r);
                                drop(buffer);
                                continue;
                            }
                            _ => unreachable!(),
                        }

                        // check sortition proof and content semantics
                        let sortition_proof = validation::check_sortition_proof(&block);
                        match sortition_proof {
                            BlockResult::Pass => {}
                            _ => {
                                warn!(
                                    "Ignoring invalid block {:.8}: {}",
                                    block.hash(),
                                    sortition_proof
                                );
                                continue;
                            }
                        }
                        let mut validity_bool: bool = false;
                        let content_semantic =
                            validation::check_content_semantic(&block, &self.chain, &self.blockdb);
                        match content_semantic {
                            BlockResult::Pass => {validity_bool=true;}
                            _ => {
                                validity_bool = false;
                                warn!(
                                    "Ignoring invalid block {:.8}: {}",
                                    block.hash(),
                                    content_semantic
                                );
                                //continue;
                            }
                        }
                        if block.header.shard_id == SHARD_ID.load(Ordering::Relaxed){
                            let serialized_validity_bool = bincode::serialize(&validity_bool).unwrap();
                            self.blockdb.insert_block_validity(&block.hash(), &serialized_validity_bool); //RR_future: Restrict to transaction blocks only
                        }

                        debug!("Processing block {:.8}", block.hash());

                        new_validated_block(
                            &block,
                            &self.mempool,
                            &self.blockdb,
                            &self.chain,
                            &self.server,
                        );
                        context_update_sig.push(match &block.content {
                            Content::Proposer(_) => ContextUpdateSignal::NewProposerBlock,
                            Content::Voter(c) => ContextUpdateSignal::NewVoterBlock(c.chain_number),
                            Content::Transaction(_) => ContextUpdateSignal::NewTransactionBlock,
                        });
                        let mut buffer = self.buffer.lock().unwrap();
                        let mut resolved_by_current = buffer.satisfy(block.hash());
                        drop(buffer);
                        if !resolved_by_current.is_empty() {
                            debug!(
                                "Resolved dependency for {} buffered blocks",
                                resolved_by_current.len()
                            );
                        }
                        for b in resolved_by_current.drain(..) {
                            to_process.push(b);
                        }
                    }
                    // tell the miner to update the context
                    for sig in context_update_sig {
                        self.context_update_chan.send(sig).unwrap();
                    }

                    if !to_request.is_empty() {
                        to_request.sort();
                        to_request.dedup();
                        peer.write(Message::GetBlocks(to_request));
                    }
                }
                Message::Bootstrap(after) => {
                    debug!("Asked for all blocks after {}", &after);
                    /*
                     * TODO: recover this message
                    for batch in self.blockdb.blocks_after(&after, 500) {
                        peer.write(Message::Blocks(batch));
                    }
                    */
                }
            }
        }
    }
}
