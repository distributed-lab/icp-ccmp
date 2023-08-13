pub mod balances;
pub mod chains;
pub mod config;
pub mod daemons;
pub mod evm_chains;
pub mod job;
pub mod messages;
pub mod pending_tx;

use candid::CandidType;
use ic_web3_rs::ic::get_public_key;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{storage_get, storage_set};
use balances::BalancesStorage;
use chains::ChainsStorage;
use job::Job;
use messages::Message;

use self::{daemons::DaemonsStorage, pending_tx::PendingTransactionsStorage};

pub const MINIMUM_CYCLES: u64 = 100_000_000_000;
pub const HTTP_OUTCALL_CYCLES_COST: u64 = 49_140_000;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("ic error: {0}")]
    IcError(String),
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default)]
pub struct Storage {
    pub key: String,
    pub public_key: String,
    pub chains_storage: ChainsStorage,
    pub signer_job: Job,
    pub writer_job: Job,
    pub checker_job: Job,
    pub listened_messages: Vec<Message>,
    pub signed_messages: Vec<Message>,
    pub balances_storage: BalancesStorage,
    pub daemon_storage: DaemonsStorage,
    pub pending_txs_storage: PendingTransactionsStorage,
}

impl Storage {
    pub async fn get_public_key() -> Result<String, StorageError> {
        let cached_public_key = storage_get!(public_key);
        if !cached_public_key.is_empty() {
            return Ok(cached_public_key);
        }

        let raw_public_key = get_public_key(Some(ic_cdk::id()), vec![], storage_get!(key))
            .await
            .map_err(StorageError::IcError)?;

        let public_key = hex::encode(raw_public_key);

        storage_set!(public_key, public_key.clone());

        Ok(public_key)
    }
}
