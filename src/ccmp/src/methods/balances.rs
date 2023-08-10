use std::str::FromStr;

use candid::candid_method;
use ethabi::ethereum_types::H256;
use ic_cdk::{
    api::call::{msg_cycles_accept, msg_cycles_available},
    query, update,
};
use ic_web3_rs::{
    ic::pubkey_to_address, transports::ICHttp, types::TransactionId, Error as Web3Error, Web3,
};

use crate::{
    log,
    types::{
        balances::{Balance, BalancesStorage},
        chains::ChainsStorageError,
        evm_chains::EvmChainsStorage,
    },
    utils::{transform_processors::call_options, u256_to_nat},
};

const DEFAULT_MAX_RESP: u64 = 500_000;
const TX_SUCCESSFUL_STATUS: u64 = 1;

#[derive(Debug, thiserror::Error)]
pub enum BalancesError {
    #[error("chain storage error: {0}")]
    ChainsStorageError(#[from] ChainsStorageError),
    #[error("chain type is not EVM")]
    ChainTypeIsNotEVM,
    #[error("balance does not exist")]
    BalanceDoesNotExist,
    #[error("invalid tx hash: {0}")]
    InvalidTxHash(String),
    #[error("web3 error: {0}")]
    Web3(#[from] Web3Error),
    #[error("tx does not exist")]
    TxDoesNotExist,
    #[error("tx is not finalized")]
    TxIsNotFinalized,
    #[error("tx without destination")]
    TxWithoutDestination,
    #[error("tx destination is not balance address")]
    TxDestinationIsNotBalanceAddress,
    #[error("nonce already used")]
    NonceAlreadyUsed,
}

#[candid_method(update)]
#[update]
async fn add_balance() -> Result<String, String> {
    _add_balance().await.map_err(|err| err.to_string())
}

#[inline]
async fn _add_balance() -> Result<String, BalancesError> {
    let caller = ic_cdk::caller();

    if let Some(balance) = BalancesStorage::get_balance(&caller) {
        return Ok(balance.public_key);
    }

    log!("[BALANCE] balance added, caller: {}", caller);

    Ok(BalancesStorage::add(&caller).await)
}

#[candid_method(update)]
#[update]
async fn add_cycles() {
    let msg_cycles = msg_cycles_available();
    if msg_cycles == 0 {
        return;
    }

    msg_cycles_accept(msg_cycles);

    let caller = ic_cdk::caller();

    BalancesStorage::add_cycles(&caller, msg_cycles.into()).await;

    log!(
        "[BALANCE] cycles added, caller: {}, cycles: {}",
        caller,
        msg_cycles
    );
}

#[candid_method(update)]
#[update]
async fn add_tokens_to_evm_chain(tx_hash: String, chain_id: u64) -> Result<(), String> {
    _add_tokens_to_evm_chain(tx_hash, chain_id)
        .await
        .map_err(|e| e.to_string())
}

#[inline]
async fn _add_tokens_to_evm_chain(tx_hash: String, chain_id: u64) -> Result<(), BalancesError> {
    let caller = ic_cdk::caller();

    let Some(balance) = BalancesStorage::get_balance(&caller) else {
        return Err(BalancesError::BalanceDoesNotExist);
    };

    let evm_chain =
        EvmChainsStorage::get_chain(chain_id).ok_or(BalancesError::ChainTypeIsNotEVM)?;

    let w3 = Web3::new(ICHttp::new(&evm_chain.rpc, Some(DEFAULT_MAX_RESP)).unwrap());

    let formatted_tx_hash =
        H256::from_str(&tx_hash).map_err(|e| BalancesError::InvalidTxHash(e.to_string()))?;

    let Some(tx_receipt) = w3
        .eth()
        .transaction_receipt(formatted_tx_hash, call_options("transform".to_string()))
        .await? else {
            return Err(BalancesError::TxDoesNotExist);
        };

    let Some(tx_status) = tx_receipt.status else {
        return Err(BalancesError::TxIsNotFinalized);
    };

    if tx_status.as_u64() != TX_SUCCESSFUL_STATUS {
        return Err(BalancesError::TxIsNotFinalized);
    }

    let raw_pub_key =
        hex::decode(balance.public_key).expect("failed to decode public key from hex");
    let balance_address =
        pubkey_to_address(&raw_pub_key).expect("failed to get address from public key");

    let Some(to) = tx_receipt.to else {
        return Err(BalancesError::TxWithoutDestination);
    };

    if to != balance_address {
        return Err(BalancesError::TxDestinationIsNotBalanceAddress);
    }

    let Some(tx) = w3
        .eth()
        .transaction(
            TransactionId::Hash(formatted_tx_hash),
            call_options("transform".to_string()),
        )
        .await? else {
            return Err(BalancesError::TxDoesNotExist);
        };

    let nonce = tx.nonce.as_u64();

    if BalancesStorage::is_used_nonce(&caller, chain_id, nonce) {
        return Err(BalancesError::NonceAlreadyUsed);
    }

    let value = u256_to_nat(tx.value);

    log!(
        "[BALANCE] tokens added, caller: {}, chain_id: {}, value: {}, nonce: {}",
        caller,
        chain_id,
        value,
        nonce
    );

    BalancesStorage::add_tokens_on_chain(&caller, chain_id, value, nonce).await;

    Ok(())
}

#[candid_method(query)]
#[query]
fn get_balance() -> Option<Balance> {
    let caller = ic_cdk::caller();

    BalancesStorage::get_balance(&caller)
}
