use std::collections::HashMap;

use candid::{CandidType, Nat, Principal};
use futures::Future;
use ic_web3_rs::ic::get_public_key;
use serde::{Deserialize, Serialize};

use crate::{storage_get, STORAGE};

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, Default)]
pub struct ChainEntry {
    pub tokens: Nat,
    pub nonce: Vec<u64>,
    pub tx_count: u64,
    pub last_block: u64,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, Default)]
pub struct Balance {
    pub public_key: String,
    pub cycles: Nat,
    pub chains_data: HashMap<u64, ChainEntry>,
}

impl Balance {
    pub fn new(public_key: String) -> Self {
        Self {
            public_key,
            ..Default::default()
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, Default)]
pub struct BalancesStorage(HashMap<Principal, Balance>);

#[allow(dead_code)]
impl BalancesStorage {
    pub fn is_exists(principal: &Principal) -> bool {
        STORAGE.with(|state| state.borrow().balances_storage.0.contains_key(principal))
    }

    pub async fn add(principal: &Principal) -> String {
        let raw_public_key = get_public_key(
            Some(ic_cdk::id()),
            vec![principal.as_slice().to_vec()],
            storage_get!(key),
        )
        .await
        .expect("should get a public key");

        let public_key = hex::encode(raw_public_key);

        STORAGE.with(|state| {
            state
                .borrow_mut()
                .balances_storage
                .0
                .insert(*principal, Balance::new(public_key.clone()));
        });

        public_key
    }

    pub async fn add_cycles(principal: &Principal, cycles: Nat) {
        if !Self::is_exists(principal) {
            Self::add(principal).await;
        }

        STORAGE.with(|state| {
            state
                .borrow_mut()
                .balances_storage
                .0
                .get_mut(principal)
                .expect("should get a balance")
                .cycles += cycles;
        });
    }

    pub async fn add_tokens_on_chain(
        principal: &Principal,
        chain_id: u64,
        tokens: Nat,
        nonce: u64,
    ) {
        if !Self::is_exists(principal) {
            Self::add(principal).await;
        }

        STORAGE.with(|state| {
            let mut state = state.borrow_mut();
            let tokens_entry = state
                .balances_storage
                .0
                .get_mut(principal)
                .expect("should get a balance")
                .chains_data
                .entry(chain_id)
                .or_insert(ChainEntry::default());

            tokens_entry.tokens += tokens;
            tokens_entry.nonce.push(nonce);
        });
    }

    pub fn get_balance(principal: &Principal) -> Option<Balance> {
        STORAGE.with(|state| state.borrow().balances_storage.0.get(principal).cloned())
    }

    pub fn reduce_cycles(principal: &Principal, cycles: Nat) {
        STORAGE.with(|state| {
            state
                .borrow_mut()
                .balances_storage
                .0
                .get_mut(principal)
                .expect("should get a balance")
                .cycles -= cycles;
        });
    }

    pub fn reduce_tokens_on_chain(principal: &Principal, chain_id: u64, tokens: Nat) {
        STORAGE.with(|state| {
            let mut state = state.borrow_mut();
            let token_entry = state
                .balances_storage
                .0
                .get_mut(principal)
                .expect("should get a balance")
                .chains_data
                .get_mut(&chain_id)
                .expect("should get a tokens entry");

            token_entry.tokens -= tokens;
        });
    }

    pub fn is_used_nonce(principal: &Principal, chain_id: u64, nonce: u64) -> bool {
        STORAGE.with(|state| {
            state
                .borrow()
                .balances_storage
                .0
                .get(principal)
                .expect("should get a balance")
                .chains_data
                .get(&chain_id)
                .expect("should get a tokens entry")
                .nonce
                .contains(&nonce)
        })
    }

    pub fn update_last_block(principal: &Principal, chain_id: u64, last_block: u64) {
        STORAGE.with(|state| {
            let mut state = state.borrow_mut();
            let token_entry = state
                .balances_storage
                .0
                .get_mut(principal)
                .expect("should get a balance")
                .chains_data
                .get_mut(&chain_id)
                .expect("should get a tokens entry");

            token_entry.last_block = last_block;
        });
    }

    pub fn increment_tx_count(principal: &Principal, chain_id: u64) -> u64 {
        STORAGE.with(|state| {
            let mut state = state.borrow_mut();
            let token_entry = state
                .balances_storage
                .0
                .get_mut(principal)
                .expect("should get a balance")
                .chains_data
                .get_mut(&chain_id)
                .expect("should get a tokens entry");

            token_entry.tx_count += 1;

            token_entry.tx_count
        })
    }

    pub fn decrement_tx_count(principal: &Principal, chain_id: u64) {
        STORAGE.with(|state| {
            let mut state = state.borrow_mut();
            let token_entry = state
                .balances_storage
                .0
                .get_mut(principal)
                .expect("should get a balance")
                .chains_data
                .get_mut(&chain_id)
                .expect("should get a tokens entry");

            token_entry.tx_count -= 1;
        });
    }

    pub async fn with_tx<F, Fut, T, E>(principal: &Principal, chain_id: u64, f: F) -> Result<T, E>
    where
        F: FnOnce(u64) -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let tx_count = Self::increment_tx_count(principal, chain_id);

        let result = f(tx_count).await;

        if result.is_err() {
            Self::decrement_tx_count(principal, chain_id);
        }

        result
    }
}
