use std::str::FromStr;

use candid::CandidType;
use ethabi::{Address, Log, Token};
use ic_cdk::api::management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId, SignWithEcdsaArgument};
use ic_web3_rs::signing::keccak256;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    chains::{Chain, ChainType},
    evm_chains::EvmChainError,
};
use crate::{
    log, storage_get,
    utils::{
        encoding, format_evm_address,
        signing::{self, get_eth_v},
        UtilsError,
    },
    STORAGE,
};

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("encoding error: {0}")]
    Encoding(#[from] encoding::EncodingError),
    #[error("sign with ecdsa error: {0}")]
    SignWithECDSAError(String),
    #[error("utils error: {0}")]
    Utils(#[from] UtilsError),
    #[error("chain does not exist")]
    ChainDoesNotExist,
    #[error("unkwnown chain type")]
    UnknownChainType,
    #[error("evm chain error: {0}")]
    EvmChain(#[from] EvmChainError),
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub enum Encoding {
    #[default]
    Plain,
    AbiEncodePacked,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct Message {
    pub index: u64,
    pub from_chain_id: u64,
    pub to_chain_id: u64,
    pub sender: Vec<u8>,
    pub message: Vec<u8>,
    pub receiver: Vec<u8>,
    pub signature: Option<Vec<u8>>,
    pub daemon_id: u64,
}

impl Message {
    pub fn new(log: Log, from_chain_id: u64, daemon_id: u64) -> Option<Self> {
        let index = log.params[0].clone().value.into_uint().unwrap().as_u64();
        let ccmp_chain_id = log.params[1].clone().value.into_uint().unwrap().as_u64();
        let sender = log.params[2]
            .clone()
            .value
            .into_address()
            .unwrap()
            .0
            .to_vec();
        let message = log.params[3].clone().value.into_bytes().unwrap();
        let receiver = log.params[4].clone().value.into_bytes().unwrap();

        let chain_metadata = STORAGE.with(|storage| {
            storage
                .borrow()
                .chains_storage
                .chains_metadata
                .get(&ccmp_chain_id)
                .cloned()
        });

        chain_metadata.as_ref()?;

        Some(Message {
            index,
            from_chain_id,
            to_chain_id: ccmp_chain_id,
            sender,
            message,
            receiver,
            daemon_id,
            ..Default::default()
        })
    }

    pub fn encode(&self, encoding: Encoding) -> Result<Vec<u8>, MessageError> {
        match encoding {
            Encoding::AbiEncodePacked => {
                let receiver = Address::from_str(&format_evm_address(hex::encode(&self.receiver))?)
                    .expect("invalid receiver address");

                let tokens = vec![
                    Token::Uint(self.index.into()),
                    Token::Uint(self.from_chain_id.into()),
                    Token::Uint(self.to_chain_id.into()),
                    Token::Bytes(self.sender.clone()),
                    Token::Bytes(self.message.clone()),
                    Token::Address(receiver),
                ];

                Ok(encoding::encode_packed(&tokens)?)
            }
            _ => Ok(self.message.clone()),
        }
    }

    pub async fn sign(self) -> Result<Self, MessageError> {
        let chain_metadata = STORAGE.with(|storage| {
            storage
                .borrow()
                .chains_storage
                .chains_metadata
                .get(&self.to_chain_id)
                .ok_or(MessageError::ChainDoesNotExist)
                .cloned()
        })?;

        let message_hash = match chain_metadata.chain_type {
            ChainType::Evm => {
                let message = self.encode(Encoding::AbiEncodePacked)?;
                keccak256(&message).to_vec()
            }
            _ => return Err(MessageError::UnknownChainType),
        };

        let sign_args = SignWithEcdsaArgument {
            message_hash: message_hash.clone(),
            derivation_path: vec![],
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: storage_get!(key),
            },
        };

        let mut signature = signing::sign(sign_args)
            .await
            .map_err(|(_, msg)| MessageError::SignWithECDSAError(msg))?
            .0
            .signature;

        match chain_metadata.chain_type {
            ChainType::Evm => {
                signature.push(get_eth_v(&signature, &message_hash));
            }
            _ => return Err(MessageError::UnknownChainType),
        }

        let mut message = self.clone();
        message.signature = Some(signature);

        Ok(message)
    }

    pub async fn send(self) -> Result<(), MessageError> {
        log!("[WRITER] sending message to chain: {}", self.to_chain_id);
        let chain_metadata = STORAGE.with(|storage| {
            storage
                .borrow()
                .chains_storage
                .chains_metadata
                .get(&self.to_chain_id)
                .ok_or(MessageError::ChainDoesNotExist)
                .cloned()
        })?;

        match chain_metadata.chain_type {
            ChainType::Evm => {
                let evm_chain = STORAGE.with(|storage| {
                    storage
                        .borrow()
                        .chains_storage
                        .evm_chains_storage
                        .0
                        .get(&self.to_chain_id)
                        .ok_or(MessageError::ChainDoesNotExist)
                        .cloned()
                })?;

                evm_chain.write(self).await?;
            }
            _ => return Err(MessageError::UnknownChainType),
        }

        Ok(())
    }
}
