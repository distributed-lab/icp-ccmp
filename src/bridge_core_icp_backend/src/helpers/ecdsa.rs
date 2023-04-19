use ic_web3::{
    ic::recover_address,
    types::H160,
};

pub fn get_eth_v(signature: &[u8], message: &[u8], public_key: &H160) -> Result<u8, String> {
    let pub_key = hex::encode(public_key);

    let mut recovered_address = recover_address(message.to_vec(), signature.to_vec(), 0);
    if recovered_address == pub_key {
        return Ok(27);
    }

    recovered_address = recover_address(message.to_vec(), signature.to_vec(), 1);
    if recovered_address == pub_key {
        return Ok(28);
    }

    Err("invalid sig, msg or pk".to_string())
}