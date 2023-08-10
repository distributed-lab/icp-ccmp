use std::collections::HashMap;

use async_trait::async_trait;
use candid::CandidType;
use ethabi::{Error as EthabiError, Token};
use ic_web3_rs::{
    contract::{Contract, Options},
    ic::pubkey_to_address,
    ic::KeyInfo,
    transports::ICHttp,
    types::{H160, U256},
    Error as Web3Error, Web3,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::chains::{Chain, ChainMetadata, ChainType};
use crate::{
    log, storage_get,
    types::{balances::BalancesStorage, daemons::DaemonsStorage, messages::Message},
    utils::{transform_processors::call_options, UtilsError},
    STORAGE,
};

const DEFAULT_MAX_RESP: u64 = 500_000;
const RECEIVER_ABI: &[u8] = include_bytes!("../assets/ReceiverABI.json");
const CCMP_CONTRACT_RECEIVER_METHOD: &str = "receiveMessage";
const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;
const EVM_ADDRESS_LENGTH: usize = 20;

#[derive(Error, Debug)]
pub enum EvmChainError {
    #[error("web3 error: {0}")]
    Web3(#[from] Web3Error),
    #[error("invalid chain id: {0}")]
    InvalidChainId(String),
    #[error("utils error: {0}")]
    Utils(#[from] UtilsError),
    #[error("ethabi error: {0}")]
    Ethabi(#[from] EthabiError),
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct EvmChain {
    pub name: String,
    pub id: u64,
    pub rpc: String,
}

impl EvmChain {
    pub async fn new(name: String, rpc: String) -> Result<Self, EvmChainError> {
        let w3 = Web3::new(ICHttp::new(&rpc, Some(DEFAULT_MAX_RESP)).unwrap());

        let chain_id = w3
            .eth()
            .chain_id(call_options("transform".to_string()))
            .await?;

        if chain_id > U256::from(u64::MAX) {
            return Err(EvmChainError::InvalidChainId(
                "chain id is too large".to_string(),
            ));
        }

        Ok(Self {
            name,
            id: chain_id.as_u64(),
            rpc,
        })
    }
}

#[async_trait]
impl Chain for EvmChain {
    type Error = EvmChainError;

    async fn write(&self, message: Message) -> Result<(), Self::Error> {
        let daemon = DaemonsStorage::get_daemon(self.id).expect("daemon not found");
        let balance = BalancesStorage::get_balance(&daemon.creator).expect("balance not found");
        let from = pubkey_to_address(&hex::decode(balance.public_key).unwrap())
            .expect("unable to get eth address from public key");
        let address = format!("0x{}", hex::encode(from.0));

        let w3 = Web3::new(ICHttp::new(&self.rpc, Some(DEFAULT_MAX_RESP)).unwrap());

        if message.receiver.len() != EVM_ADDRESS_LENGTH {
            return Ok(());
        }

        let receiver = H160::from_slice(&message.receiver);

        let ccmp_contract = Contract::from_json(w3.eth(), receiver, RECEIVER_ABI)?;

        let mut gas_price = w3
            .eth()
            .gas_price(call_options("transform".to_string()))
            .await?;

        gas_price = (gas_price / 10) * 12;

        let mut options = Options::with(|op| {
            op.gas_price = Some(gas_price);
        });

        let key_info = KeyInfo {
            derivation_path: vec![daemon.creator.as_slice().to_vec()],
            key_name: storage_get!(key),
            ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
        };

        let params = vec![
            Token::Uint(U256::from(message.index)),
            Token::Uint(U256::from(message.from_chain_id)),
            Token::Uint(U256::from(message.to_chain_id)),
            Token::Bytes(message.sender),
            Token::Bytes(message.message),
            Token::Address(receiver),
            Token::Bytes(message.signature.unwrap_or_default()),
        ];

        let tx_hash = BalancesStorage::with_tx(
            &daemon.creator,
            message.to_chain_id,
            |tx_count| async move {
                options.nonce = Some(U256::from(tx_count));
                ccmp_contract
                    .signed_call(
                        CCMP_CONTRACT_RECEIVER_METHOD,
                        &params,
                        options,
                        address,
                        key_info,
                        self.id,
                    )
                    .await
            },
        )
        .await?;

        log!(
            "[WRITER] message sent to evm chain, id: {}, tx hash: 0x{}",
            message.to_chain_id,
            hex::encode(tx_hash.0)
        );

        Ok(())
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct EvmChainsStorage(pub HashMap<u64, EvmChain>);

impl EvmChainsStorage {
    pub fn add(evm_chain: EvmChain) -> u64 {
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();

            let index = storage.chains_storage.chains_count;
            storage.chains_storage.chains_metadata.insert(
                index,
                ChainMetadata::new(evm_chain.name.clone(), ChainType::Evm),
            );
            storage
                .chains_storage
                .evm_chains_storage
                .0
                .insert(index, evm_chain);
            storage.chains_storage.chains_count += 1;

            index
        })
    }

    pub fn get_chain(id: u64) -> Option<EvmChain> {
        STORAGE.with(|storage| {
            let storage = storage.borrow();

            storage
                .chains_storage
                .evm_chains_storage
                .0
                .get(&id)
                .cloned()
        })
    }
}
