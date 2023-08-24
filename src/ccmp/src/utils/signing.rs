use candid::Principal;
use ic_cdk::api::{
    call::{call_with_payment, CallResult},
    management_canister::ecdsa::{SignWithEcdsaArgument, SignWithEcdsaResponse},
};
use libsecp256k1::{recover, Message, RecoveryId, Signature};

use crate::storage_get;

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;

pub fn get_eth_v(sig: &[u8], msg: &[u8]) -> u8 {
    let message = Message::parse_slice(msg).expect("invalid message");
    let signature = Signature::parse_overflowing_slice(sig).expect("invalid signature");
    let recovery_id = RecoveryId::parse(0).expect("invalid recovery id");

    let rec_pub_key = recover(&message, &signature, &recovery_id).expect("unable to recover");
    let pub_key = hex::decode(storage_get!(public_key)).expect("invalid public key");
    if pub_key == rec_pub_key.serialize_compressed() {
        return 27;
    }

    28
}

pub async fn sign(args: SignWithEcdsaArgument) -> CallResult<(SignWithEcdsaResponse,)> {
    call_with_payment(
        Principal::management_canister(),
        "sign_with_ecdsa",
        (args,),
        ECDSA_SIGN_CYCLES,
    )
    .await
}
