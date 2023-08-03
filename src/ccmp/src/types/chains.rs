use std::collections::HashMap;

use async_trait::async_trait;
use candid::CandidType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::STORAGE;

use super::{evm_chains::EvmChainsStorage, messages::Message};

#[derive(Error, Debug)]
pub enum ChainsStorageError {
    #[error("unknown chain type")]
    UnknownChainType,
    #[error("chain not found")]
    ChainNotFound,
}

#[async_trait]
pub trait Chain {
    type Error;

    async fn listen(id: u64) -> Result<(), Self::Error>;
    async fn write(&self, message: Message) -> Result<(), Self::Error>;
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub enum ChainType {
    #[default]
    Unknown,
    Evm,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct ChainMetadata {
    pub name: String,
    pub chain_type: ChainType,
}

impl ChainMetadata {
    pub fn new(name: String, chain_type: ChainType) -> Self {
        Self { name, chain_type }
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct ChainsStorage {
    pub chains_count: u64,
    pub chains_metadata: HashMap<u64, ChainMetadata>,
    pub evm_chains_storage: EvmChainsStorage,
}

impl ChainsStorage {
    pub fn remove_chain(id: u64) -> Result<(), ChainsStorageError> {
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            let chain_metadata = storage
                .chains_storage
                .chains_metadata
                .get(&id)
                .ok_or(ChainsStorageError::ChainNotFound)?;

            match chain_metadata.chain_type {
                ChainType::Evm => {
                    storage.chains_storage.chains_metadata.remove(&id);
                    storage.chains_storage.evm_chains_storage.0.remove(&id);

                    Ok(())
                }
                _ => Err(ChainsStorageError::UnknownChainType),
            }
        })
    }

    pub fn get_chain_metadata(id: u64) -> Result<ChainMetadata, ChainsStorageError> {
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            let chain_metadata = storage
                .chains_storage
                .chains_metadata
                .get(&id)
                .ok_or(ChainsStorageError::ChainNotFound)?;

            Ok(chain_metadata.clone())
        })
    }

    pub fn get_chains_metadata() -> Result<HashMap<u64, ChainMetadata>, ChainsStorageError> {
        STORAGE.with(|storage| {
            let storage = storage.borrow();
            let chains_metadata = storage.chains_storage.chains_metadata.clone();

            Ok(chains_metadata)
        })
    }
}
