use log::info;
use rand::Rng;
use ring::signature;
use std::ops::Add;
use std::time;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::network::server::Handle as ServerHandle;
use crate::types::key_pair;
use crate::types::transaction;
use crate::network::message::Message;
use crate::types::transaction::{Transaction, SignedTransaction, Mempool};
use crate::types::address::Address;
use crate::types::hash::Hashable;
use ring::signature::{Ed25519KeyPair, KeyPair};
use ring::rand::SystemRandom;




#[derive(Clone)]
pub struct TransactionGenerator {
    mempool: Arc<Mutex<Mempool>>, 
    server: ServerHandle,
    key_pair: Arc<Ed25519KeyPair>,
}

impl TransactionGenerator {
    pub fn new(mempool: Arc<Mutex<Mempool>>, server: ServerHandle, key_pair: Arc<Ed25519KeyPair>,) -> Self {
        Self {mempool, server, key_pair,}
    }

    pub fn start(self, theta: u64) {
        thread::Builder::new()
            .name("transaction-generator".to_string())
            .spawn(move || {
                self.generate_transactions(theta);
            })
            .unwrap();
        info!("Transaction generator started");
    }


    fn generate_transactions(&self, theta: u64) {
        let mut nonce = 0;
        loop {
            //info!("NONCE: {}", nonce);
            //unimplemented!();
            if let Some(transaction) = self.create_valid_transaction(nonce) {
                nonce += 1;
                let tx_hash = transaction.hash();

                {
                    let mut mempool = self.mempool.lock().unwrap();
                    if let Err(e) = mempool.add_transaction(transaction.clone()) {
                        info!("Failed to add transaction to mempool: {}", e);
                        drop(mempool);
                        continue;
                    }

                    self.server.broadcast(Message::NewTransactionHashes(vec![tx_hash]));
                    /*info!(
                        "Generated, added to mempool, and broadcasted new transaction with hash: {:?}", 
                        tx_hash
                    );*/
                    
                    drop(mempool);
                }

            } else {
                info!("Failed to generate a valid transaction.");
            }

            if theta != 0 {
                //let interval = time::Duration::from_millis(10 * theta);
                let interval = time::Duration::from_millis(2 * theta);
                thread::sleep(interval);
            }

        }
        
    }

    fn create_valid_transaction(&self, nonce: u64) -> Option<SignedTransaction> {
        let sender_address = Address::from_public_key_bytes(self.key_pair.public_key().as_ref());

        let mut rng = rand::thread_rng();
        //info!("Sender Addy: {}", sender_address);

        // Generate random receiver and transfer amount

        let receiver = self.generate_random_address();
        let value = rng.gen_range(1..10); // Small amount between 1 and 10

        //info!("Receiver Addy: {}", receiver);
        //info!("Amount Going to Receiver: {}", value);


        // Create transaction
        let transaction = Transaction {
            receiver,
            value,
            nonce, 
        };

        // Sign transaction
        let signature = self.key_pair.sign(&bincode::serialize(&transaction).unwrap());

        Some(SignedTransaction {
            transaction,
            signature: signature.as_ref().to_vec(),
            public_key: self.key_pair.public_key().as_ref().to_vec(),
        })

    }

    fn generate_random_address(&self) -> Address {
        // Generate 32 random bytes to simulate a public key
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
        Address::from_public_key_bytes(&random_bytes)
    }


    fn create_random_transactions(&self) -> SignedTransaction {
        let receiver = self.generate_random_address();
        let value = rand::thread_rng().gen_range(1..1000);
        let nonce = rand::thread_rng().gen_range(1..1000);

        let transaction = Transaction {
            receiver,
            value,
            nonce,
        };

        // Generate a key pair and sign the transaction
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate Ed25519 key");
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).expect("Failed to parse Ed25519 key");
        let signature = key_pair.sign(&bincode::serialize(&transaction).unwrap());

        SignedTransaction {
            transaction,
            signature: signature.as_ref().to_vec(),
            public_key: key_pair.public_key().as_ref().to_vec(),
        }
    }

}




