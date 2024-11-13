use log::info;
use rand::Rng;
use std::ops::Add;
use std::time;
use std::thread;
use std::sync::{Arc, Mutex};
//use crate::mempool::Mempool;
use crate::network::server::Handle as ServerHandle;
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

}

impl TransactionGenerator {
    pub fn new(mempool: Arc<Mutex<Mempool>>, server: ServerHandle) -> Self {
        Self {mempool, server}
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
        loop {
            //unimplemented!();

            // Generate a random transaction
            let transaction = self.create_random_transactions();

            // Lock mempool and add transaction
            {
                let mut mempool = self.mempool.lock().unwrap();
                if let Err(e) = mempool.add_transaction(transaction.clone()) {
                    info!("Failed to add transaction to mempool: {}", e);
                }
                drop(mempool);
            }

            // Broadcast the new transaction hash
            let tx_hash = transaction.hash();
            self.server.broadcast(Message::NewTransactionHashes(vec![tx_hash]));

            info!("Generated and broadcasted new transaction with hash: {:?}", tx_hash);


            if theta != 0 {
                //let interval = time::Duration::from_millis(10 * theta);
                let interval = time::Duration::from_millis(2 * theta);
                thread::sleep(interval);
            }
        }
    }

    fn generate_random_address(&self) -> Address {
        // Generate 32 random bytes to simulate a public key
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
        Address::from_public_key_bytes(&random_bytes)
    }


    fn create_random_transactions(&self) -> SignedTransaction {
        let receiver = self.generate_random_address();
        let value = rand::thread_rng().gen_range(1..1000);
        let account_nonce = rand::thread_rng().gen_range(1..1000);

        let transaction = Transaction {
            receiver,
            value,
            account_nonce,
        };

        // Generate a key pair and sign the transaction
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate Ed25519 key");
        //let key = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).expect("Failed to parse Ed25519 key");

        /* 
        let key: Ed25519KeyPair = Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Failed to generate Ed25519 key")
            .into();*/
        
        //let key = Ed25519KeyPair::generate_pkcs8(&mut rand::thread_rng()).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).expect("Failed to parse Ed25519 key");
        let signature = key_pair.sign(&bincode::serialize(&transaction).unwrap());

        SignedTransaction {
            transaction,
            signature: signature.as_ref().to_vec(),
            public_key: key_pair.public_key().as_ref().to_vec(),
        }
    }

}
