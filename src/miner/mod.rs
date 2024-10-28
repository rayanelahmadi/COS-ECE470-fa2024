pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use rand::Rng;
use std::time;

use std::thread;

use crate::blockchain;
use crate::types::block::{Block, Header, Content};
use crate::blockchain::Blockchain;
use crate::types::hash::{Hashable, H256};
use std::sync::{Arc, Mutex};

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
    blockchain: Arc<Mutex<Blockchain>>, // thread-safe blockchain access 
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(blockchain: &Arc<Mutex<Blockchain>>) -> (Context, Handle, Receiver<Block>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();



    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain: Arc::clone(blockchain),
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    new(&blockchain)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
         
        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO for student: actual mining, create a block
            // TODO for student: if block mining finished, you can have something like: self.finished_block_chan.send(block.clone()).expect("Send finished block error");

            if let OperatingState::Run(lambda) = self.operating_state {

                // Retrieve the latest parent block's hash (tip)
                
                let parent_hash = {
                    let blockchain = self.blockchain.lock().unwrap();
                    blockchain.tip()
                };

                // Mining: trying random nonces until a solution is found 
                let mut nonce = rand::thread_rng().gen::<u32>();
                let timestamp = time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                //let difficulty = H256::from([0x0f; 32]);
                let difficulty = hex_literal::hex!("00001fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();

                let block = Block {
                    header: Header {
                        parent: parent_hash,
                        nonce,
                        difficulty, // Use static difficulty from genesis block
                        timestamp,
                        merkle_root: H256::from([0u8; 32]), // Placeholder for real content
                    },
                    content: Content { transactions: vec![] }, // Placeholder content
                };
                
                // Proof-of-Work check
                if block.hash() <= difficulty {
                    // Send mined block to channel 
                    self.finished_block_chan
                        .send(block.clone())
                        .expect("Send finished block error");
                    info!("Block succesfully mined with nonce: {}", nonce);

                    // Insert block into blockchain and update tip & Update the parent hash to the newly mined block                    
                    {
                        let mut blockchain = self.blockchain.lock().unwrap();
                        blockchain.insert(&block);
                    }
                }

                if lambda != 0 {
                    let interval = time::Duration::from_micros(lambda as u64);
                    thread::sleep(interval);
                }


            }

            /* 
            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }*/
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::hash::Hashable;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST

#[test]
fn custom_miner_ten_blocks() {
    use std::time::{Instant, Duration};

    // Initialize miner with a simple blockchain
    let (miner_ctx, miner_handle, finished_block_chan) = test_new();
    miner_ctx.start();
    miner_handle.start(0); // Set lambda to 0 for maximum mining speed

    // Start timer
    let start_time = Instant::now();
    
    let mut mined_blocks = Vec::new();
    while mined_blocks.len() < 10 {
        if let Ok(block) = finished_block_chan.recv_timeout(Duration::from_secs(10)) {
            mined_blocks.push(block);
        } else {
            panic!("Failed to mine 10 blocks within the time limit");
        }
    }

    // Ensure that we are within the 1-minute time limit
    let elapsed_time = start_time.elapsed();
    assert!(elapsed_time <= Duration::from_secs(60), "Mining took too long");

    // Verify the chain
    for i in 1..mined_blocks.len() {
        assert_eq!(mined_blocks[i - 1].hash(), mined_blocks[i].get_parent());
    }

    println!("Successfully mined 10 blocks in {:?}", elapsed_time);
}

#[test]
fn custom_block_validity_check_test() {
    use std::time::Duration;
    // Initialize miner with a moderate difficulty
    let moderate_difficulty = H256::from([0xff; 32]);
    let (miner_ctx, miner_handle, finished_block_chan) = test_new();
    miner_ctx.start();
    miner_handle.start(0); // Lambda set to 0 for continuous mining

    // Mine 5 blocks
    let mut mined_blocks = Vec::new();
    for _ in 0..5 {
        if let Ok(block) = finished_block_chan.recv_timeout(Duration::from_secs(5)) {
            // Check that the block meets the moderate difficulty requirement
            assert!(block.hash() <= moderate_difficulty, "Block does not meet difficulty requirement");
            mined_blocks.push(block);
        } else {
            panic!("Failed to mine a block within the expected time");
        }
    }

    // Ensure all blocks are linked correctly (parent-child)
    for i in 1..mined_blocks.len() {
        assert_eq!(mined_blocks[i - 1].hash(), mined_blocks[i].get_parent(), "Blocks are not correctly linked");
    }

    println!("Successfully mined and validated {} blocks", mined_blocks.len());
}


