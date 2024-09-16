use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, UnparsedPublicKey, ED25519};

use rand::Rng;
use bincode;

// Define an Address type 
pub type Address = Vec<u8>;

// Define Transaction struct with sender, receiver, value fields
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub sender: Address,
    pub receiver: Address,
    pub value: u64,
}

// Define SignedTransaction struct with transaction, signature, public_key fields
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
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
    (0..32).map(|_| rng.gen()).collect()
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    //unimplemented!()
    Transaction {
        sender: generate_random_address(),
        receiver: generate_random_address(),
        value: rand::thread_rng().gen_range(1..1000), 
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
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST