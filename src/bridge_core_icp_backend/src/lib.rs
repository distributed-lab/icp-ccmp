mod tests;
mod helpers;
mod networks;

use candid::Nat;
use ethabi::{
    Token,
    Address,
    Uint,    
};
use ic_cdk::api::management_canister::{
    ecdsa::{
        sign_with_ecdsa,
        SignWithEcdsaArgument,
        EcdsaKeyId,
        EcdsaCurve
    },
    http_request::{
        HttpResponse,
        TransformArgs
    }
};
use ic_web3::{
    Web3,
    types::{
        H256,
        Log
    },
    ic::get_eth_addr,
    transports::ICHttp,
    signing::{
        hash_message,
        keccak256
    },
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use crate::{
    helpers::{
        encoding::{
            encode_packed,
            nat_to_normal_str,
            get_deposit_native_event,
            DepositedNativeEvent
        },
        ecdsa::get_eth_v
    },
    networks::Network
};

type Result<T, E> = std::result::Result<T, E>;
type NetworkStore = BTreeMap<String, String>;

const KEY_NAME: &str = "dfx_test_key";
const INPURA_API_KEY: &str = "d009354476b140008dd04c741c00341b";

thread_local!{
    static NETWORK_STORE: RefCell<NetworkStore> = RefCell::default();
}

#[ic_cdk_macros::query]
fn transform(response: TransformArgs) -> HttpResponse {
    response.response
}

#[ic_cdk_macros::update]
async fn eth_address() -> Result<String, String> {
    match get_eth_addr(None, None, KEY_NAME.to_string()).await {
        Ok(eth_addr) => Ok(hex::encode(eth_addr)),
        Err(err) => Err(err)
    }
}

#[ic_cdk_macros::update]
async fn add_network(network: String, address: String) {
    NETWORK_STORE.with(|store| {
        store
            .borrow_mut()
            .insert(network, address);
    })
}

#[ic_cdk_macros::update]
async fn get_erc20_redeem_signature(
    token_address: String,
    tx_hash: String,
    nonce: Nat,
    is_wrapped: bool,
    network: String
) -> Result<String, String> {
    let node_url = format!(
        "https://{}.infura.io/v3/{}",
        Network::from(&network).as_str(),
        INPURA_API_KEY
    );

    let w3 = Web3::new(
        ICHttp::new(&node_url, None, None).unwrap()
    );

    let decoded_hash = hex::decode(tx_hash)
        .map_err(|err| format!("failed to decode the tx hash: {}", err))?;

    let tx_receipt = w3.eth()
        .transaction_receipt(H256::from_slice(&decoded_hash))
        .await
        .map_err(|err| format!("failed to get a tx receipt: {}", err))?
        .ok_or_else(|| return "the tx doesn't exist or wasn't indexed yet".to_string())?;

    if tx_receipt.status.unwrap_or_default().is_zero() {
        return Err("The tx has failed".to_string());
    }

    let event = check_logs(&network, &tx_receipt.logs)
        .map_err(|err| format!("failed to check logs: {}", err))?;

    let token_address = Address::from_slice(&hex::decode(&token_address)
        .map_err(|err| format!("failed to decode the token address: {}", err))?);
    let receiver = Address::from_slice(&hex::decode(&event.receiver)
        .map_err(|err| format!("failed to decode the receiver: {}", err))?);
    let nonce = Uint::from_dec_str(&nat_to_normal_str(&nonce))
        .map_err(|err| format!("failed to decode the nonce: {}", err))?;
    let chain_id = Network::from(&event.network).chain_id();

    let data = vec![
        Token::Address(token_address),
        Token::Uint(event.amount),
        Token::Address(receiver),
        Token::FixedBytes(decoded_hash.clone()),
        Token::Uint(nonce),
        Token::Uint(chain_id),
        Token::Bool(is_wrapped)
    ];
    
    let encoded_data = encode_packed(&data)
        .map_err(|err| format!("failed to perform a packing encoding: {}", err))?;

    let msg_hash = hash_message(&keccak256(&encoded_data)).as_bytes().to_vec();

    let call_args = SignWithEcdsaArgument {
        message_hash: msg_hash.clone(),
        derivation_path: vec![ic_cdk::id().as_slice().to_vec()],
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: KEY_NAME.to_string(),
        }
    };
    
    let mut signature = match sign_with_ecdsa(call_args).await {
        Ok((response,)) => response.signature,
        Err((rejection_code, msg)) => {
            return Err(format!("failed to sign a hash. Rejection code: {:?}, msg: {:?}", rejection_code, msg));
        }
    };

    let public_key = get_eth_addr(None, None, KEY_NAME.to_string())
        .await
        .map_err(|err| format!("failed to get an eth address: {}", err))?;

    let v: u8 = get_eth_v(&signature, &msg_hash, &public_key)
        .map_err(|err| format!("failed to get a v: {}", err))?;

    signature.push(v);

    Ok(format!("0x{}", hex::encode(signature)))
}

fn check_logs(network: &str, logs: &[Log]) -> Result<DepositedNativeEvent, String> {
    let contract: Option<String> = NETWORK_STORE.with(|store| {
        let store = store.borrow();

        let contract = store.get(network);

        if contract.is_none() {
            return None;
        }

        Some(contract.unwrap().to_string())
    });


    let contract = contract
        .ok_or_else(|| return "The network is not supported".to_string())?;

    let log = logs.first()
        .ok_or_else(|| return "The tx doesn't contain any logs".to_string())?;

    if hex::encode(log.address).to_lowercase() != contract.to_lowercase() {
        return Err("Invalid contract address".to_string());
    }    

    let event = get_deposit_native_event(&log)
        .ok_or_else(|| return "failed to get an event".to_string())?;

    Ok(event)
}
