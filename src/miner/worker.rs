use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::types::block::{Block, Content, Header};
use crate::network::server::Handle as ServerHandle;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::types::hash::{Hashable, H256};
use crate::network::message::Message;
use crate::types::transaction::{Mempool, SignedTransaction};
use::std::time;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>, // Thread-safe blockchain reference 
    mempool: Arc<Mutex<Mempool>>, // Thread-safe Mempool reference
    max_transactions_per_block: usize, // Transaction limit per block
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>,
        mempool: &Arc<Mutex<Mempool>>,
        max_transactions_per_block: usize,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain),
            mempool: Arc::clone(mempool),
            max_transactions_per_block,
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let block = self.finished_block_chan.recv().expect("Receive finished block error");
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            {
                let mut blockchain = self.blockchain.lock().unwrap();
                blockchain.insert(&block);
                drop(blockchain);
                //info!("IN WORKER");
            }

            info!("Block inserted into blockchain with hash: {:?}", block.hash());

            // Broadcast the newly mined block's hash to the network
            let new_block_hash = block.hash();
            self.server.broadcast(Message::NewBlockHashes(vec![new_block_hash]));

            info!("Broadcasted new block hash: {:?}", new_block_hash);

            // Remove transactions included in this block from the mempool
            let mut mempool = self.mempool.lock().unwrap();
            let tx_hashes: Vec<_> = block.content.transactions.iter().map(|tx| tx.hash()).collect();
            mempool.remove_transactions(tx_hashes);
            drop(mempool);
            /* 
            for tx in block.content.transactions {
                info!("Noce Removed: {}", tx.transaction.nonce);
            }*/
            }
    }
    /* 
    // Function to create a new block with transactions from the mempool
    fn create_blcok(&self, parent_hash: H256) -> Block {
        let mut mempool = self.mempool.lock().unwrap();
        let transactions = mempool.get_transactions_for_block(self.max_transactions_per_block);
        drop(mempool);

        let timestamp = time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

        let header = Header {
            parent: parent_hash,
            nonce: 0, // will be updated during mining
            difficulty: H256::from([0xff; 32]), // Use your actual difficulty here
            timestamp,
            merkle_root: crate::types::merkle::MerkleTree::new(&transactions).root(),

        };

        let content = Content { transactions };

        Block {header, content}
    }*/
}
