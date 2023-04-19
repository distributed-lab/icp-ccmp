use candid::Nat;
use thiserror::Error;
use ethabi::{
    Token,
    Uint,
    Event,
    EventParam,
    ParamType,
    RawLog,
    ethereum_types::H256
};

use ic_web3::types::Log;

#[derive(Debug, Error)]
pub enum EncodePackedError {
    #[error("This token cannot be encoded in packed mode: {0:?}")]
    InvalidToken(Token),

    #[error("FixedBytes token length > 32")]
    InvalidBytesLength,
}

pub fn nat_to_normal_str(nat: &Nat) -> String {
    nat.to_string().chars().filter(|&c| c != '_').collect()
}

pub fn encode_packed(tokens: &[Token]) -> Result<Vec<u8>, EncodePackedError> {
    let mut max = 0;
    for token in tokens {
        check(token)?;
        max += max_encoded_length(token);
    }

    let mut bytes = Vec::with_capacity(max);
    for token in tokens {
        encode_token(token, &mut bytes, false);
    }
    Ok(bytes)
}

fn max_encoded_length(token: &Token) -> usize {
    match token {
        Token::Int(_) | Token::Uint(_) | Token::FixedBytes(_) => 32,
        Token::Address(_) => 20,
        Token::Bool(_) => 1,
        Token::Array(vec) | Token::FixedArray(vec) | Token::Tuple(vec) => {
            vec.iter().map(|token| max_encoded_length(token).max(32)).sum()
        }
        Token::Bytes(b) => b.len(),
        Token::String(s) => s.len(),
    }
}

fn check(token: &Token) -> Result<(), EncodePackedError> {
    match token {
        Token::FixedBytes(vec) if vec.len() > 32 => Err(EncodePackedError::InvalidBytesLength),

        Token::Tuple(_) => Err(EncodePackedError::InvalidToken(token.clone())),
        Token::Array(vec) | Token::FixedArray(vec) => {
            for t in vec.iter() {
                if t.is_dynamic() || matches!(t, Token::Array(_)) {
                    return Err(EncodePackedError::InvalidToken(token.clone()))
                }
                check(t)?;
            }
            Ok(())
        }

        _ => Ok(()),
    }
}

fn encode_token(token: &Token, out: &mut Vec<u8>, in_array: bool) {
    match token {
        Token::Address(addr) => {
            if in_array {
                out.extend_from_slice(&[0; 12]);
            }
            out.extend_from_slice(&addr.0)
        }
        Token::Int(n) | Token::Uint(n) => {
            let mut buf = [0; 32];
            n.to_big_endian(&mut buf);
            out.extend_from_slice(&buf);
        }
        Token::Bool(b) => {
            if in_array {
                out.extend_from_slice(&[0; 31]);
            }
            out.push((*b) as u8);
        }
        Token::FixedBytes(bytes) => {
            out.extend_from_slice(bytes);
            if in_array {
                let mut remaining = vec![0; 32 - bytes.len()];
                out.append(&mut remaining);
            }
        }

        Token::Bytes(bytes) => out.extend_from_slice(bytes),
        Token::String(s) => out.extend_from_slice(s.as_bytes()),
        Token::Array(vec) | Token::FixedArray(vec) => {
            for token in vec {
                encode_token(token, out, true);
            }
        }

        token => unreachable!("Uncaught invalid token: {token:?}"),
    }
}

pub struct DepositedNativeEvent {
    pub amount: Uint,
    pub receiver: String,
    pub network: String,
}

lazy_static::lazy_static! {
    static ref DEPOSITED_NATIVE_EVENT: Event = Event {
        name: "DepositedNative".to_string(),
        inputs: vec![
            EventParam {
                name: "amount".to_string(),
                kind: ParamType::Uint(256),
                indexed: false,
            },
            EventParam {
                name: "receiver".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
            EventParam {
                name: "network".to_string(),
                kind: ParamType::String,
                indexed: false,
            },
        ],
        anonymous: false,
    };
}

pub fn get_deposit_native_event(log: &Log) -> Option<DepositedNativeEvent> {
    let topics: Vec<H256> = log.topics.iter().map(|t| H256::from_slice(t.as_bytes())).collect();

    let raw_log = RawLog {
        topics,
        data: log.data.0.clone(),
    };

    let parsed_log = DEPOSITED_NATIVE_EVENT.parse_log(raw_log)
        .ok()?;

    DepositedNativeEvent {
        amount: parsed_log.params[0].value.clone().into_uint().unwrap(),
        receiver: parsed_log.params[1].value.clone().into_string().unwrap()[2..].to_string(),
        network: parsed_log.params[2].value.clone().into_string().unwrap(),
    }.into()
}
