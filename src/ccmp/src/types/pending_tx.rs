use std::str::FromStr;

use candid::{CandidType, Nat, Principal};
use ethabi::ethereum_types::H256;
use ic_web3_rs::{transports::ICHttp, Error as Web3Error, Web3};
use scopeguard::defer;
use serde::{Deserialize, Serialize};

use crate::{
    log,
    types::{daemons::Daemon, messages::Message},
    utils::{transform_processors::call_options, u256_to_nat},
    STORAGE,
};

use super::{
    balances::BalancesStorage,
    chains::{ChainType, ChainsStorage},
    daemons::DaemonsStorage,
    evm_chains::EvmChainsStorage,
    HTTP_OUTCALL_CYCLES_COST, MINIMUM_CYCLES,
};

const EVM_CHECKER_HTTP_OUTCALLS_COUNT: u64 = 1;
const CHECKER_JOB_EXECTUTION_COST: u64 = 2_000_000;

#[derive(Debug, thiserror::Error)]
pub enum PendingTransactionError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] Web3Error),
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, Default)]
pub struct PendingTransaction {
    pub tx_hash: String,
    pub message: Message,
    pub gas_price: Nat,
}

impl PendingTransaction {
    pub fn new(tx_hash: String, message: Message, gas_price: Nat) -> Self {
        Self {
            tx_hash,
            message,
            gas_price,
        }
    }

    pub async fn check(self) -> Result<bool, PendingTransactionError> {
        let chain_metadata = ChainsStorage::get_chain_metadata(self.message.to_chain_id)
            .expect("Chain metadata not found");

        match chain_metadata.chain_type {
            ChainType::Evm => self.check_evm().await,
            _ => panic!("Unsupported chain type"),
        }
    }

    pub async fn check_evm(&self) -> Result<bool, PendingTransactionError> {
        let daemon = DaemonsStorage::get_daemon(self.message.daemon_id).expect("Daemon not found");
        defer! {
            Self::collect_checking_cycles(daemon.id, &daemon.creator)
        }
        let evm_chain =
            EvmChainsStorage::get_chain(self.message.to_chain_id).expect("EVM chain not found");

        let w3 = Web3::new(ICHttp::new(&evm_chain.rpc, None).unwrap());

        let tx_hash = H256::from_str(&self.tx_hash).expect("invalid tx hash");

        let tx = w3
            .eth()
            .transaction_receipt(tx_hash, call_options("transform".to_string()))
            .await?;

        let Some(tx) = tx else {
            return Ok(false);
        };

        let used_gas = u256_to_nat(tx.gas_used.expect("used gas not found"));

        BalancesStorage::reduce_tokens_on_chain(
            &daemon.creator,
            self.message.to_chain_id,
            used_gas * self.gas_price.clone(),
        );

        Ok(true)
    }

    pub fn collect_checking_cycles(daemon_id: u64, principal: &Principal) {
        let mut used_cycles = 0;
        used_cycles += HTTP_OUTCALL_CYCLES_COST * EVM_CHECKER_HTTP_OUTCALLS_COUNT;
        used_cycles += CHECKER_JOB_EXECTUTION_COST;

        BalancesStorage::reduce_cycles(principal, Nat::from(used_cycles));

        let balance = BalancesStorage::get_balance(principal).expect("Balance not found");
        if balance.cycles < MINIMUM_CYCLES {
            log!("[DAEMONS] insufficient cycles, principal: {}", principal);
            Daemon::stop(daemon_id);
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, Default)]
pub struct PendingTransactionsStorage(pub Vec<PendingTransaction>);

impl PendingTransactionsStorage {
    pub fn add(pending_tx: PendingTransaction) {
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();

            storage.pending_txs_storage.0.push(pending_tx);
        })
    }
}
