use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::hash::H256;
use crate::blockchain::Blockchain;
use crate::types::block::Block;
use crate::types::hash::Hashable;
use std::collections::HashMap;

use log::{debug, warn, error};
use stderrlog::new;

use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;

#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>, // Add blockchain for thread-safe access
    orphan_buffer: Arc<Mutex<HashMap<H256, Vec<Block>>>>, // Orphan buffer to handle blocks with missing parents
}


impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
            orphan_buffer: Arc::new(Mutex::new(HashMap::new())), // Initialize orphan buffer
        }
    }

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
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }

                Message::NewBlockHashes(hashes) => {

                    let blockchain = self.blockchain.lock().unwrap();

                    // Request blocks we don't already have in blockchain
                    // Filter out hashes that are not already in the blockchain (check all blocks)
                    let missing_hashes: Vec<H256> = hashes
                        .into_iter()
                        .filter(|hash| !blockchain.blocks.contains_key(hash))
                        .collect();

                    drop (blockchain);

                    if !missing_hashes.is_empty() {
                        peer.write(Message::GetBlocks(missing_hashes));
                    }
                }

                Message::GetBlocks(hashes) => {
                    let blockchain = self.blockchain.lock().unwrap();
                    let blocks_to_send: Vec<_> = hashes
                        .into_iter()
                        .filter_map(|hash| blockchain.blocks.get(&hash).cloned())
                        .collect();
                    drop(blockchain);

                    if !blocks_to_send.is_empty() {
                        peer.write(Message::Blocks(blocks_to_send));
                    }
                }

                Message::Blocks(blocks) => {
                    let mut blockchain = self.blockchain.lock().unwrap();
                    let mut new_block_hashes = Vec::new();

                    for block in blocks {
                        let block_hash = block.hash();
                        //debug!("Received new block with hash: {:?}", block_hash);

                        // Check PoW Validity
                        if block_hash > block.header.difficulty {
                            debug!("Block with hash {:?} failed PoW check", block_hash);
                            continue;
                        }

                        // Check if parent exists in blockchain 
                        let parent_hash = block.header.parent;
                        if !blockchain.blocks.contains_key(&parent_hash) {
                            debug!("Parent block missing for block {:?}", block_hash);

                            // Add block to orphan buffer
                            self.orphan_buffer.lock().unwrap().entry(parent_hash)
                                .or_insert_with(Vec::new)
                                .push(block.clone());

                            // Request the missing parent
                            peer.write(Message::GetBlocks(vec![parent_hash]));
                            continue;
                        }

                        // Difficulty check with parent block
                        let parent_block = blockchain.blocks.get(&parent_hash).unwrap();
                        if block.header.difficulty != parent_block.header.difficulty {
                            debug!("Block with hash {:?} has incorrect difficulty", block_hash);
                            continue;
                        }

                        // Insert block and add to broadcast if new
                        if !blockchain.blocks.contains_key(&block_hash) {
                            blockchain.insert(&block);
                            new_block_hashes.push(block_hash);
                        }
                    }

                    drop(blockchain);

                    if !new_block_hashes.is_empty() {
                        self.server.broadcast(Message::NewBlockHashes(new_block_hashes));
                    }

                    // Process any orphans that may now have their parent
                    self.process_orphans();
                }
                _=> unimplemented!(),
            }
        }
    }

    fn process_orphans(&self) {
        let mut processed_any = true;
        while processed_any {
            processed_any = false;
            let mut orphan_buffer = self.orphan_buffer.lock().unwrap();
            let mut blockchain = self.blockchain.lock().unwrap();
            let mut new_block_hashes = Vec::new();

            // Process any orphans whose parents now exist in the blockchain
            for (parent_hash, orphans) in orphan_buffer.clone().iter() {
                if blockchain.blocks.contains_key(parent_hash) {
                    for orphan in orphans {
                        let orphan_hash = orphan.hash();
                        blockchain.insert(orphan);
                        new_block_hashes.push(orphan_hash);
                        processed_any = true;

                    }

                    // Remove processed orphans from buffer
                    orphan_buffer.remove(parent_hash);

                }
            }

            drop(blockchain);
            drop(orphan_buffer);

            // Broadcast newly processed orphan blocks
            if !new_block_hashes.is_empty() {
                self.server.broadcast(Message::NewBlockHashes(new_block_hashes));
            }

            
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();

    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let worker = Worker::new(1, msg_chan, &server, &blockchain);
    worker.start(); 

    let chain_hashes = blockchain.lock().unwrap().all_blocks_in_longest_chain();

    (test_msg_sender, server_receiver, chain_hashes)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }



}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST