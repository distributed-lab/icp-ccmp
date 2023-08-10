pub mod encoding;
pub mod signing;
pub mod transform_processors;

use std::num::ParseIntError;

use candid::Nat;
use ic_web3_rs::types::U256;
use num_bigint::BigUint;
use sha3::{Digest, Keccak256};
use thiserror::Error;

const EVM_ADDR_PREFIX: &str = "0x";
const EVM_ADDR_LEN: usize = 40;

#[derive(Error, Debug)]
pub enum UtilsError {
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    #[error("parsing int error: {0}")]
    ParseInt(#[from] ParseIntError),
}

pub fn format_evm_address(addr: String) -> Result<String, UtilsError> {
    let addr = addr.trim_start_matches(EVM_ADDR_PREFIX);
    if addr.len() != EVM_ADDR_LEN || !addr.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(UtilsError::InvalidAddress("invalid hex".into()));
    }

    let lower_addr = addr.to_ascii_lowercase();

    let mut hasher = Keccak256::new();
    hasher.update(lower_addr);
    let hash = hex::encode(hasher.finalize());

    let mut checksum = String::new();
    for (i, char) in hash.chars().enumerate() {
        if i > 39 {
            break;
        }
        if u32::from_str_radix(&char.to_string()[..], 16)? > 7 {
            checksum.push_str(&addr[i..i + 1].to_ascii_uppercase());
            continue;
        }

        checksum.push_str(&addr[i..i + 1].to_ascii_lowercase());
    }

    Ok(format!("0x{checksum}"))
}

pub fn u256_to_nat(num: U256) -> Nat {
    let mut buffer = vec![];
    num.to_big_endian(&mut buffer);

    Nat(BigUint::from_bytes_be(&buffer))
}
