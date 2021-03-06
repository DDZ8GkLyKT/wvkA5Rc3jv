use prism::crypto::hash::H256;
use prism::transaction::{Output, Transaction};
use prism::visualization;
use prism::{self, blockchain, blockdb, miner::memory_pool};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

const NUM_VOTER_CHAINS: u16 = 3;

fn main() {
    // initialize all sorts of stuff
    let blockdb_path = std::path::Path::new("/tmp/prism_itest_self_mining.rocksdb");
    let blockdb = blockdb::BlockDatabase::new(blockdb_path).unwrap();
    let blockdb = Arc::new(blockdb);

    let (state_update_sink, state_update_source) = mpsc::channel();
    let blockchain = blockchain::BlockChain::new(NUM_VOTER_CHAINS, state_update_sink);
    let blockchain = Arc::new(Mutex::new(blockchain));

    let mempool = memory_pool::MemoryPool::new();
    let mempool = Arc::new(Mutex::new(mempool));

    let peer_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let peer_port = 12345;
    let peer_addr = std::net::SocketAddr::new(peer_ip, peer_port);

    let (_server, miner, _wallet) =
        prism::start(peer_addr, &blockdb, &blockchain, &mempool).unwrap();

    let vis_ip = "127.0.0.1".parse::<std::net::IpAddr>().unwrap();
    let vis_port = 8888;
    let vis_addr = std::net::SocketAddr::new(vis_ip, vis_port);
    visualization::Server::start(vis_addr, Arc::clone(&blockchain));

    /*
    // insert a fake key into the wallet
    let our_addr: H256 = (&[0; 32]).into();
    wallet.add_key(our_addr);

    // fund-raising
    let funding = Transaction {
        input: vec![],
        output: vec![Output {
            value: 1000000,
            recipient: our_addr,
        }],
        signatures: vec![],
    };
    wallet.add_transaction(&funding);
    assert_eq!(wallet.balance(), 1000000);

    // send some money to outself
    wallet.send_coin(our_addr, 5000);
    // the transaction has not been mined, so our balance will dip for now
    assert_eq!(wallet.balance(), 0);
    */

    // mine a block
    for _ in 0..50 {
        miner.step();
    }
    std::thread::sleep(std::time::Duration::from_millis(2000));
    miner.exit();

    loop {
        std::thread::park();
    }
}
