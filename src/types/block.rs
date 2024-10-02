use ring::digest::SHA256;
use serde::{Serialize, Deserialize};
use crate::types::hash::{H256, Hashable};
use crate::types::merkle::MerkleTree;
use crate::types::transaction::SignedTransaction;
use std::time::{SystemTime, UNIX_EPOCH};



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub content: Content,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
   pub transactions: Vec<SignedTransaction>,
}


impl Hashable for Block {
    fn hash(&self) -> H256 {
        //unimplemented!()
        self.header.hash()
    }
}

impl Hashable for Header {
    fn hash(&self) -> H256 {
        // Serialize header and hash it using H256 function
        let serialized_header = bincode::serialize(&self).expect("Serialization should not fail");
        H256::from(ring::digest::digest(&ring::digest::SHA256, &serialized_header))
    }
}
impl Block {
    pub fn get_parent(&self) -> H256 {
        //unimplemented!()
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        //unimplemented!()
        self.header.difficulty
    }
}

impl Content {
    pub fn new(transactions: Vec<SignedTransaction>) -> Self {
        Content { transactions }
    }
}

impl Header {
    pub fn new(parent: H256, nonce: u32, difficulty: H256,timestamp: u128, merkle_root: H256) -> Self {
        Header {
            parent,
            nonce,
            difficulty,
            timestamp,
            merkle_root
        }
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    //unimplemented!()
    let nonce: u32 = rand::random(); // Generate a random nonce
    let difficulty = H256::from([0xff; 32]); // Set a high difficulty (all bits set to 1)
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis(); // Get current UNIX timestamp in milliseconds

    let transactions = Vec::new(); // Empty content for now
    let merkle_root = MerkleTree::new(&transactions).root(); // Generate Merkle root of empty input
    
    let header = Header::new(*parent, nonce, difficulty, timestamp, merkle_root);
    let content = Content::new(transactions);

    Block { header, content }
}