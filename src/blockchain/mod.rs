use crate::types::block::Block;
use crate::types::hash::H256;
use std::collections::HashMap;
use crate::types::block::{Header, Content};
use crate::types::hash::Hashable;
use crate::types::transaction::SignedTransaction;
pub struct Blockchain {
    pub blocks: HashMap<H256, Block>, // Store blocks by their hash
    heights: HashMap<H256, usize>, // Store heights of each block
    tip: H256, // Keep track of the last block's hash (tip of longest chain)
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // Create a genesis block with fixed values for the fields
        let genesis_block = Block {
            // Define the genesis block's header and content 
            header: Header {
                parent: H256::from([0x00; 32]),
                nonce: 0,
                //difficulty: H256::from([0x0f; 32]), //[0xff; 32]
                difficulty: hex_literal::hex!("00001fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into(),
                timestamp: 0,
                merkle_root: H256::from([0x00; 32]),
            },
            content: Content{
                transactions: vec![],
            },
        };

        let genesis_hash = genesis_block.hash();

        let mut blocks = HashMap::new();
        blocks.insert(genesis_hash, genesis_block);

        let mut heights = HashMap::new();
        heights.insert(genesis_hash, 0); // Genesis block is at height 0

        Self {
            blocks,
            heights,
            tip: genesis_hash, // Genesis block is the tip at creation
        }

    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        //unimplemented!()
        let block_hash = block.hash();
        let parent_hash = block.get_parent();

        // Ensure parent block is already in the blockchain
        if let Some(parent_height) = self.heights.get(&parent_hash) {
            // Insert the block into the blockchain
            self.blocks.insert(block_hash, block.clone());

            // Compute the height of the new block (parent height + 1)
            let block_height = parent_height + 1;
            self.heights.insert(block_hash, block_height);

            // Update the tip if the new block extends the longest chain
            if block_height > *self.heights.get(&self.tip).unwrap() {
                self.tip = block_hash;
            }
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        //unimplemented!()
        self.tip
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        // unimplemented!()
        //vec![]
        let mut chain = Vec::new();
        let mut current_hash = self.tip;

        // Traverse backward from the tip to the genesis block 
        while let Some(block) = self.blocks.get(&current_hash) {
            chain.push(current_hash);
            current_hash = block.get_parent();
            if current_hash == H256::from([0x00; 32]) {
                break;
            }
        }

        // Reverse to order from genesis to tip
        chain.reverse();
        chain

    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use ring::signature::{Ed25519KeyPair, KeyPair};

    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;
    use crate::types::merkle::MerkleTree;
    use crate::types::transaction::{generate_random_transaction, sign};

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }
    // Custom Tests to Verfiy Correctness
    #[test]
    fn CUSTOM_test_forking_and_longest_chain() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
    
        // Generate and insert first block
        let block_1 = generate_random_block(&genesis_hash);
        blockchain.insert(&block_1);
        assert_eq!(blockchain.tip(), block_1.hash());
    
        // Fork 1: Add a block to the first block
        let block_2a = generate_random_block(&block_1.hash());
        blockchain.insert(&block_2a);
        assert_eq!(blockchain.tip(), block_2a.hash());
    
        // Fork 2: Add a block to the first block (another fork)
        let block_2b = generate_random_block(&block_1.hash());
        blockchain.insert(&block_2b);
        assert_eq!(blockchain.tip(), block_2a.hash()); // The tip should still be block_2a because it's the first block in this level
    
        // Extend the second fork further (making it the longest chain)
        let block_3b = generate_random_block(&block_2b.hash());
        blockchain.insert(&block_3b);
        assert_eq!(blockchain.tip(), block_3b.hash()); // Now the tip should be block_3b because it's the longest chain
    }

    #[test]
    fn CUSTOM_test_inserting_multiple_blocks() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();

        let mut last_hash = genesis_hash;

        // Insert 50 blocks sequentially
        for _ in 0..50 {
            let new_block = generate_random_block(&last_hash);
            blockchain.insert(&new_block);
            last_hash = new_block.hash();
        }

        // Ensure the tip is the last block
        assert_eq!(blockchain.tip(), last_hash);

        // Ensure all blocks are part of the longest chain
        let all_blocks = blockchain.all_blocks_in_longest_chain();
        assert_eq!(all_blocks.len(), 51); // Genesis + 50 blocks
    }

    #[test]
    fn CUSTOM_test_out_of_order_insertion() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();

        // Insert a child block first (shouldn't be possible, but let's try)
        let block_1 = generate_random_block(&genesis_hash);
        let block_2 = generate_random_block(&block_1.hash());

        blockchain.insert(&block_2); // Shouldn't extend the chain as the parent block isn't there yet
        assert_eq!(blockchain.tip(), genesis_hash); // Tip should still be genesis

        // Now insert the parent block
        blockchain.insert(&block_1);
        assert_eq!(blockchain.tip(), block_1.hash()); // Tip should now be block_1

        // Finally insert block_2
        blockchain.insert(&block_2);
        assert_eq!(blockchain.tip(), block_2.hash()); // Tip should now be block_2
    }

    #[test]
    fn CUSTOM_test_fork_with_multiple_extensions() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();

        // First branch
        let block_1 = generate_random_block(&genesis_hash);
        let block_2 = generate_random_block(&block_1.hash());
        blockchain.insert(&block_1);
        blockchain.insert(&block_2);
        
        assert_eq!(blockchain.tip(), block_2.hash()); // Tip should be block_2

        // Fork branch off block_1
        let block_fork = generate_random_block(&block_1.hash());
        let block_fork2 = generate_random_block(&block_fork.hash());
        blockchain.insert(&block_fork);
        blockchain.insert(&block_fork2);
        
        assert_eq!(blockchain.tip(), block_fork2.hash()); // Tip should now be block_fork2 as it extends longer
    }

    #[test]
    fn CUSTOM_test_block_with_signed_transactions() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();

        // Generate random transactions
        let transaction_1 = generate_random_transaction();
        let transaction_2 = generate_random_transaction();

        // Create a key pair for signing
        let key = Ed25519KeyPair::from_seed_unchecked(&[0u8; 32]).unwrap();
        
        // Sign both transactions with the same key
        let signature_1 = sign(&transaction_1, &key);
        let signature_2 = sign(&transaction_2, &key);

        // Create two SignedTransaction objects with unique transactions and signatures
        let signed_tx_1 = SignedTransaction {
            transaction: transaction_1,
            signature: signature_1.as_ref().to_vec(),
            public_key: key.public_key().as_ref().to_vec(),
        };

        let signed_tx_2 = SignedTransaction {
            transaction: transaction_2,
            signature: signature_2.as_ref().to_vec(),
            public_key: key.public_key().as_ref().to_vec(),
        };

        // Add transactions to the block content
        let transactions = vec![signed_tx_1.clone(), signed_tx_2.clone()];

        // Create a block with these transactions
        let block_with_tx = Block {
            header: Header {
                parent: genesis_hash,
                nonce: rand::random(),
                difficulty: H256::from([0xff; 32]),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis(),
                merkle_root: MerkleTree::new(&transactions).root(), // Merkle root from transactions
            },
            content: Content {
                transactions,
            },
        };

        // Insert the block into the blockchain
        blockchain.insert(&block_with_tx);

        // Check that the tip is updated correctly
        assert_eq!(blockchain.tip(), block_with_tx.hash());

        // Check that the block contains the correct merkle root
        let block_from_chain = blockchain.blocks.get(&block_with_tx.hash()).unwrap();
        assert_eq!(block_with_tx.header.merkle_root, block_from_chain.header.merkle_root);
    }

    



}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST