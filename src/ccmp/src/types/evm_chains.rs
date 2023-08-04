use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use candid::CandidType;
use ethabi::{Error as EthabiError, Event, EventParam, ParamType, RawLog, Token};
use ic_web3_rs::{
    contract::{Contract, Options},
    ic::pubkey_to_address,
    ic::KeyInfo,
    transports::ICHttp,
    types::{BlockNumber, FilterBuilder, H160, H256, U256},
    Error as Web3Error, Web3,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::chains::{Chain, ChainMetadata, ChainType};
use crate::{
    log, storage_get,
    types::messages::Message,
    utils::{format_evm_address, transform_processors::call_options, UtilsError},
    STORAGE,
};

lazy_static! {
    pub static ref MESSAGE_EVENT: Event = Event {
        name: "CcmpMessage".into(),
        inputs: vec![
            EventParam {
                name: "index".into(),
                kind: ParamType::Uint(256),
                indexed: true,
            },
            EventParam {
                name: "ccmp_chain_id".into(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
            EventParam {
                name: "sender".into(),
                kind: ParamType::Address,
                indexed: false,
            },
            EventParam {
                name: "message".into(),
                kind: ParamType::Bytes,
                indexed: false,
            },
            EventParam {
                name: "receiver".into(),
                kind: ParamType::Bytes,
                indexed: false,
            },
        ],
        anonymous: false,
    };
    pub static ref MESSAGE_EVENT_SIGNATURE: H256 = MESSAGE_EVENT.signature();
}

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
    #[error("chain now found")]
    ChainNotFound,
    #[error("ethabi error: {0}")]
    Ethabi(#[from] EthabiError),
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct EvmChain {
    pub name: String,
    id: u64,
    rpc: String,
    ccmp_contract_addr: String,
    block_number: u64,
}

impl EvmChain {
    pub async fn new(
        name: String,
        rpc: String,
        ccmp_contract_addr: String,
    ) -> Result<Self, EvmChainError> {
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

        let block_number = w3
            .eth()
            .block_number(call_options("transform".to_string()))
            .await?
            .as_u64();

        Ok(Self {
            name,
            id: chain_id.as_u64(),
            rpc,
            ccmp_contract_addr: format_evm_address(ccmp_contract_addr)?,
            block_number,
        })
    }
}

#[async_trait]
impl Chain for EvmChain {
    type Error = EvmChainError;

    async fn listen(id: u64) -> Result<(), Self::Error> {
        log!("[LISTENER] listening chain, id: {id}");
        let evm_chain = EvmChainsStorage::get_chain(id).ok_or(EvmChainError::ChainNotFound)?;
        let w3 = Web3::new(ICHttp::new(&evm_chain.rpc, Some(DEFAULT_MAX_RESP)).unwrap());

        let block_number = w3
            .eth()
            .block_number(call_options("transform".to_string()))
            .await?
            .as_u64();

        let filter = FilterBuilder::default()
            .from_block(BlockNumber::Number(evm_chain.block_number.into()))
            .to_block(BlockNumber::Number(block_number.into()))
            .address(vec![H160::from_str(&evm_chain.ccmp_contract_addr).unwrap()])
            .build();

        let logs = w3
            .eth()
            .logs(filter, call_options("transform".to_string()))
            .await?;

        if logs.is_empty() {
            log!(
                "[LISTENER] listening chain finished, id: {}, no messages",
                id
            );
            EvmChainsStorage::update_block_number(id, block_number);
            return Ok(());
        }

        let parsed_logs = logs
            .into_iter()
            .filter(|log| log.topics[0] == *MESSAGE_EVENT_SIGNATURE)
            .map(|log| {
                MESSAGE_EVENT.parse_log(RawLog {
                    topics: log.topics,
                    data: log.data.0,
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let mut messages = vec![];
        for log in parsed_logs {
            if let Some(message) = Message::new(log, id) {
                messages.push(message);
                continue;
            }
        }

        log!(
            "[LISTENER] listening chain finished, id: {}, produced messages number: {}",
            id,
            messages.len()
        );

        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.listened_messages.append(&mut messages)
        });

        let block_number = w3
            .eth()
            .block_number(call_options("transform".to_string()))
            .await?
            .as_u64();

        EvmChainsStorage::update_block_number(id, block_number);
        Ok(())
    }

    async fn write(&self, message: Message) -> Result<(), Self::Error> {
        let public_key = storage_get!(public_key);
        let from = pubkey_to_address(&hex::decode(public_key).unwrap())
            .expect("unable to get eth address from public key");
        let address = format!("0x{}", hex::encode(from.0));

        let w3 = Web3::new(ICHttp::new(&self.rpc, Some(DEFAULT_MAX_RESP)).unwrap());

        if message.receiver.len() != EVM_ADDRESS_LENGTH {
            return Ok(());
        }

        let receiver = H160::from_slice(&message.receiver);

        let ccmp_contract = Contract::from_json(w3.eth(), receiver, RECEIVER_ABI)?;

        let tx_count = w3
            .eth()
            .transaction_count(from, None, call_options("transform".to_string()))
            .await?;

        let mut gas_price = w3
            .eth()
            .gas_price(call_options("transform".to_string()))
            .await?;

        gas_price = (gas_price / 10) * 12;

        let options = Options::with(|op| {
            op.nonce = Some(tx_count);
            op.gas_price = Some(gas_price);
        });

        let key_info = KeyInfo {
            derivation_path: vec![],
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

        let tx_hash = ccmp_contract
            .signed_call(
                CCMP_CONTRACT_RECEIVER_METHOD,
                &params,
                options,
                address,
                key_info,
                self.id,
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

    pub fn update_block_number(id: u64, block_number: u64) {
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();

            if let Some(evm_chain) = storage.chains_storage.evm_chains_storage.0.get_mut(&id) {
                evm_chain.block_number = block_number;
            }
        })
    }
}
