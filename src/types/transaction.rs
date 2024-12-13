use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, UnparsedPublicKey, ED25519, KeyPair};
use crate::types::hash::{Hashable, H256};
use crate::types::address::Address; // Import Address from address.rs

use rand::Rng;
use bincode;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::info;


// Define Transaction struct with sender, receiver, value fields
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    //pub sender: Address,
    pub receiver: Address,
    pub value: u64,
    pub nonce: u64, // Used in state.rs
}

// Define SignedTransaction struct with transaction, signature, public_key fields
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl SignedTransaction {
    // Get sender address by deriving it from the public key
    pub fn sender_address(&self) -> Address {
        Address::from_public_key_bytes(&self.public_key)
    }
}

impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        let serialized_tx = bincode::serialize(self).expect("Serialization should not fail");
        H256::from(ring::digest::digest(&ring::digest::SHA256, &serialized_tx))
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    //unimplemented!()

    // Serialize transaction using bincode
    let serialized_transaction = bincode::serialize(t).expect("Failed to serialize transaction");

    // Sign transaction with provided key 
    key.sign(&serialized_transaction)

}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    //unimplemented!()

    // Serialize transaction using bincode
    let serialized_transaction = bincode::serialize(t).expect("Failed to serialize transaction");

    // Create public key verifier
    let public_key = UnparsedPublicKey::new(&ED25519, public_key);

    // Verify signature
    public_key.verify(&serialized_transaction, signature).is_ok()
}

// Custom Helper Method
fn generate_random_address() -> Address {

    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect(); // 32 random bytes
    Address::from_public_key_bytes(&random_bytes) // Use random bytes like a public key
}




#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    //unimplemented!()
    Transaction {
        //sender: generate_random_address(),
        receiver: generate_random_address(),
        value: rand::thread_rng().gen_range(1..1000), 
        nonce: rand::thread_rng().gen_range(1..1000),
    }
}

pub struct Mempool {
    pool: HashMap<H256, SignedTransaction>, // Store transactions by their hash
    max_size: usize, // Max number of transactions allowed 
}

impl Mempool {
    // Create a new Mempool with a size limit 
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: HashMap::new(),
            max_size,
        }

    }

    // Add a transaction to the mempool if it passes validity checks 
    pub fn add_transaction(&mut self, tx: SignedTransaction) -> Result<(), &'static str> {
        if self.pool.len() >= self.max_size {
            return Err("Mempool is full");
        }

        // Ensure transaction is not already in mempool
        let tx_hash = tx.hash();

        if self.pool.contains_key(&tx_hash) {
            return Err("Duplicate transaction");
        }

        // Verify signature 
        if !verify(&tx.transaction, &tx.public_key, &tx.signature) {
            return Err("Invalid Signature");
        }
        
        // Add transaction to the mempool
        self.pool.insert(tx_hash, tx);
        Ok(())
    }

    // Remove transactions from the mempool that are already in a block
    pub fn remove_transactions(&mut self, tx_hashes: Vec<H256>) {
        for hash in tx_hashes {
            self.pool.remove(&hash);
            //info!("Mempool Size: {}", self.pool.len());
        }

    }

    // Get all transactions for block mining up to the limit
    pub fn get_transactions_for_block(&self, limit: usize) -> Vec<SignedTransaction> {
        self.pool.values().cloned().take(limit).collect()
    }

    pub fn contains_transactions(&self, tx_hash: &H256) -> bool {
        self.pool.contains_key(tx_hash)
    }

    pub fn get_transactions(&self, tx_hash: &H256) -> Option<SignedTransaction> {
        self.pool.get(tx_hash).cloned()
    }

    pub fn get_all_transactions(&self) -> Vec<SignedTransaction> {
        self.pool.values().cloned().collect()
    }

    pub fn update_with_state(&mut self, state: &crate::types::state::State) {
        let invalid_tx_hashes: Vec<H256> = self
            .pool
            .values()
            .filter(|tx|!state.is_valid_transaction(tx))
            .map(|tx|tx.hash())
            .collect();
        self.remove_transactions(invalid_tx_hashes);
    }
    
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }

    #[test]
    fn add_and_remove_transaction_from_mempool() {
        let mut mempool = Mempool::new(10);

        let transaction = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&transaction, &key);
        let signed_tx = SignedTransaction {
            transaction,
            signature: signature.as_ref().to_vec(),
            public_key: key.public_key().as_ref().to_vec(),
        };

        // Add transaction to the mempool
        assert!(mempool.add_transaction(signed_tx.clone()).is_ok());

        // Ensure transaction is in the mempool
        assert!(mempool.pool.contains_key(&signed_tx.hash()));

        // Remove transaction
        mempool.remove_transactions(vec![signed_tx.hash()]);

        // Ensure transaction is removed
        assert!(!mempool.pool.contains_key(&signed_tx.hash()));
    }


}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST