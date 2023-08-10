use std::time::Duration;

use candid::{candid_method, CandidType};
use ic_cdk::{query, update};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::{Validate, ValidationErrors};

use crate::{
    log,
    types::{
        balances::BalancesStorage,
        chains::{ChainType, ChainsStorage},
        daemons::{Daemon, DaemonsStorage},
    },
    STORAGE,
};

lazy_static! {
    static ref EVM_ADDRESS_REGEX: Regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
}

// TODO: calculate more precisely
const MINIMUM_CYCLES: u64 = 100_000_000_000;

#[derive(Error, Debug)]
pub enum DaemonsError {
    #[error("chain not found")]
    ChainNotFound,
    #[error("balance not found")]
    BalanceNotFound,
    #[error("invalid ccmp contract address")]
    InvalidCcmpContractAddress,
    #[error("validation erros: {0}")]
    ValidationErrors(#[from] ValidationErrors),
    #[error("daemon not found")]
    DaemonNotFound,
    #[error("not the creator of this daemon")]
    NotDaemonCreator,
    #[error("insufficient cycles")]
    InsufficientCycles,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, Default, Validate)]
pub struct RegisterDaemonArgs {
    pub listen_chain_id: u64,
    pub ccmp_contract: String,
    #[validate(range(min = 1, max = 3600))]
    pub interval_in_secs: u64,
}

#[candid_method(update)]
#[update]
pub fn register_daemon(args: RegisterDaemonArgs) -> Result<u64, String> {
    _register_daemon(args).map_err(|e| e.to_string())
}

#[inline]
pub fn _register_daemon(args: RegisterDaemonArgs) -> Result<u64, DaemonsError> {
    args.validate()?;

    let caller = ic_cdk::caller();

    let Some(balance) = BalancesStorage::get_balance(&caller) else {
        return Err(DaemonsError::BalanceNotFound);
    };

    if balance.cycles < MINIMUM_CYCLES {
        return Err(DaemonsError::InsufficientCycles);
    }

    let Some(chain_metadata) = ChainsStorage::get_chain_metadata(args.listen_chain_id) else {
        return Err(DaemonsError::ChainNotFound);
    };

    if !is_valid_ccmp_contract(&args.ccmp_contract, chain_metadata.chain_type) {
        return Err(DaemonsError::InvalidCcmpContractAddress);
    }

    let id = DaemonsStorage::add_daemon(
        args.listen_chain_id,
        args.ccmp_contract,
        Duration::from_secs(args.interval_in_secs),
        caller,
    );

    Daemon::start(id);

    log!("[DAEMONS] registered daemon, id: {}", id);

    Ok(id)
}

fn is_valid_ccmp_contract(ccmp_contract: &str, chain_type: ChainType) -> bool {
    match chain_type {
        ChainType::Evm => EVM_ADDRESS_REGEX.is_match(ccmp_contract),
        _ => panic!("unknown chain type"),
    }
}

#[candid_method(query)]
#[query]
fn get_daemon(id: u64) -> Option<Daemon> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();

        let caller = ic_cdk::caller();

        let daemon = storage.daemon_storage.daemons.get(&id).cloned();
        if let Some(daemon) = daemon {
            if daemon.creator != caller {
                return None;
            }

            return Some(daemon);
        };

        None
    })
}

#[candid_method(query)]
#[query]
fn get_daemons() -> Vec<Daemon> {
    STORAGE.with(|storage| {
        let storage = storage.borrow();

        let caller = ic_cdk::caller();

        storage
            .daemon_storage
            .daemons
            .values()
            .filter(|daemon| daemon.creator == caller)
            .cloned()
            .collect::<Vec<_>>()
    })
}

#[candid_method(update)]
#[update]
fn start_daemon(id: u64) -> Result<(), String> {
    _start_daemon(id).map_err(|e| e.to_string())
}

#[inline]
fn _start_daemon(id: u64) -> Result<(), DaemonsError> {
    let caller = ic_cdk::caller();

    let Some(balance) = BalancesStorage::get_balance(&caller) else {
        return Err(DaemonsError::BalanceNotFound);
    };

    if balance.cycles < MINIMUM_CYCLES {
        return Err(DaemonsError::InsufficientCycles);
    }

    let Some(daemon) = DaemonsStorage::get_daemon(id) else {
        return Err(DaemonsError::DaemonNotFound);
    };

    if daemon.creator != caller {
        return Err(DaemonsError::NotDaemonCreator);
    }

    Daemon::start(id);

    Ok(())
}

#[candid_method(update)]
#[update]
fn stop_daemon(id: u64) -> Result<(), String> {
    _stop_daemon(id).map_err(|e| e.to_string())
}

#[inline]
fn _stop_daemon(id: u64) -> Result<(), DaemonsError> {
    let caller = ic_cdk::caller();

    let Some(daemon) = DaemonsStorage::get_daemon(id) else {
        return Err(DaemonsError::DaemonNotFound);
    };

    if daemon.creator != caller {
        return Err(DaemonsError::NotDaemonCreator);
    }

    Daemon::stop(id);

    Ok(())
}
