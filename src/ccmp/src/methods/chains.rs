use std::collections::HashMap;

use candid::candid_method;
use ic_cdk::{api::is_controller, query, update};
use thiserror::Error;

use crate::{
    log,
    types::{
        chains::{ChainMetadata, ChainsStorage, ChainsStorageError},
        evm_chains::{EvmChain, EvmChainError, EvmChainsStorage},
    },
};

#[derive(Error, Debug)]
enum ChainsError {
    #[error("evm chain error: {0}")]
    EvmChain(#[from] EvmChainError),
    #[error("caller is not a controller")]
    CallerIsNotAController,
    #[error("chains storage error: {0}")]
    ChainsStorage(#[from] ChainsStorageError),
}

#[candid_method(update)]
#[update]
async fn add_evm_chain(
    name: String,
    rpc: String,
    ccmp_contract_addr: String,
) -> Result<u64, String> {
    _add_evm_chain(name, rpc, ccmp_contract_addr)
        .await
        .map_err(|e| e.to_string())
}

async fn _add_evm_chain(
    name: String,
    rpc: String,
    ccmp_contract_addr: String,
) -> Result<u64, ChainsError> {
    if !is_controller(&ic_cdk::caller()) {
        return Err(ChainsError::CallerIsNotAController);
    }

    let evm_chain = EvmChain::new(name, rpc, ccmp_contract_addr).await?;

    let id = EvmChainsStorage::add(evm_chain);

    log!("[CHAINS] evm chain added, id: {}", id);

    Ok(id)
}

#[candid_method(update)]
#[update]
async fn remove_chain(id: u64) -> Result<(), String> {
    _remove_chain(id).await.map_err(|e| e.to_string())
}

async fn _remove_chain(id: u64) -> Result<(), ChainsError> {
    if !is_controller(&ic_cdk::caller()) {
        return Err(ChainsError::CallerIsNotAController);
    }

    ChainsStorage::remove_chain(id)?;

    log!("[CHAINS] chain removed, id: {}", id);

    Ok(())
}

#[candid_method(query)]
#[query]
fn get_chain_metadata(id: u64) -> Result<ChainMetadata, String> {
    ChainsStorage::get_chain_metadata(id).map_err(|e| e.to_string())
}

#[candid_method(query)]
#[query]
fn get_chains_metadata() -> Result<HashMap<u64, ChainMetadata>, String> {
    ChainsStorage::get_chains_metadata().map_err(|e| e.to_string())
}
