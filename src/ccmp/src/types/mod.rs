pub mod chains;
pub mod config;
pub mod evm_chains;
pub mod messages;

use candid::CandidType;
use ic_web3_rs::ic::get_public_key;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{storage_get, storage_set};
use chains::ChainsStorage;

use self::messages::Message;

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
    pub listener_interval_secs: u64,
    pub signer_interval_secs: u64,
    pub writer_interval_secs: u64,
    pub listened_messages: Vec<Message>,
    pub signed_messages: Vec<Message>,
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
