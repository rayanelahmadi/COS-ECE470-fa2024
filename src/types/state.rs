use std::{collections::HashMap, hash::Hash};
use ring::signature::{Ed25519KeyPair, KeyPair, ED25519};
use stderrlog::new;

use crate::types::transaction::SignedTransaction;
use crate::types::address::Address;
use log::info;

use super::transaction;

#[derive(Debug, Clone)]
pub struct State {
    // HashMap to store account: (nonce, balance)
    pub accounts: HashMap<Address, (u64, u64)>, // Address -> (nonce, balance)
}

impl State {
    // Initialize state with an ICO (Initial Coin Offering)
    pub fn new(seed: &[u8; 32]) -> Self {
        //Self {}
        let mut state = State {
            accounts: HashMap::new(),
        };

        // Initial Coin Offering (ICO): Create one account with a large balance
        let keypair = Ed25519KeyPair::from_seed_unchecked(seed).unwrap();
        let ico_address = Address::from_public_key_bytes(keypair.public_key().as_ref());
        state.accounts.insert(ico_address, (0, 1_000_000_000)); // Nonce = 0, Balance = 1,000,000
        state
    }

    pub fn is_valid_transaction(&self, tx: &SignedTransaction) -> bool {
        let sender = tx.sender_address();

        if let Some((nonce, balance)) = self.accounts.get(&sender) {
            // Check nonce matches and balance is sufficient
            //info!("BALANCE: {}", *balance);
            //info!("VALUE: {}", tx.transaction.value);
            /* 
            if (*nonce + 1 != tx.transaction.nonce || *balance < tx.transaction.value) {
                info!("{}", *nonce+1);
                info!("{}", tx.transaction.nonce);
                info!("{}", balance);
            }*/
            //info!("{}, {}", *nonce+1, tx.transaction.nonce);
            *nonce + 1 == tx.transaction.nonce && *balance >= tx.transaction.value 
        } else {
            false // Sender account not found or insufficent balance
        }
    }


    // Apply a transaction to update the state
    pub fn apply_transaction(&mut self, tx: &SignedTransaction) {
        let sender = tx.sender_address();
        let receiver = tx.transaction.receiver;

        // Update sender account
        if let Some((nonce, balance)) = self.accounts.get_mut(&sender) {
            *nonce += 1; // Increment nonce
            *balance -= tx.transaction.value; // Deduct value
            //info!("After Apply: Sender Nonce {}", nonce);
        }

        // Update or create receiver account
        self.accounts
            .entry(receiver)
            .and_modify(|(_, balance)| *balance += tx.transaction.value) // Update balance if exists
            .or_insert((0, tx.transaction.value)); // Create new account with initial balance

    }


    // Get a copy of the current state (for debugging or serialization)
    pub fn get_state_snapshot(&self) -> HashMap<Address, (u64, u64)> {
        self.accounts.clone()
    }
    
}
