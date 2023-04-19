#[cfg(test)]
mod tests {
    use ethabi::{Token, Address, Uint};
    use ic_web3::signing::{keccak256, hash_message};
    use ic_web3::types::H160;

    use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};

    use crate::{
        helpers::encoding::encode_packed,
        helpers::ecdsa::get_eth_v
    };

    const ECDSA_HEX_PRIVATE_KEY: &[u8] = "63dd6406ece1e459644301db883394a238ba4f1a2ff54249da7551f20bc2f8a9".as_bytes();

    lazy_static::lazy_static! {
        static ref TOKEN_ADDRESS: Address = Address::from_slice(&hex::decode("ac74c64A7cFdBb33c33D2827569FE6EaF9a677dB").unwrap());
        static ref AMOUNT: Uint = Uint::from(100000);
        static ref RECEIVER: Address = Address::from_slice(&hex::decode("E86C4A45C1Da21f8838a1ea26Fc852BD66489ce9").unwrap());
        static ref NONCE: Uint = Uint::from(0);
        static ref CHAID_ID: Uint = Uint::from(11155111);
        static ref TX_HASH: Vec<u8> = hex::decode("5edcd76efb884194fc1f7d348ffc4ef93c611e3ffa89aca3a2dcf0131e2844df").unwrap();
        static ref ECDSA_HEX_PUBLIC_KEY: H160 = H160::from_slice(hex::decode("267e0Dbd45866a85aA6aA39a7f696B3ff4b34d87").unwrap().as_slice());
    }

    #[test]
    fn hash() {
        let data = Token::Tuple(vec![
            Token::Address(*TOKEN_ADDRESS),
            Token::Uint(*AMOUNT),
            Token::Address(*RECEIVER),
            Token::FixedBytes((*TX_HASH.clone()).to_vec()),
            Token::Uint(*NONCE),
            Token::Uint(*CHAID_ID),
            Token::Bool(true)
        ]);
        
        let encoded_data = ethabi::encode(&[data]);
        println!("Encoded data: {}", hex::encode(&encoded_data));
        
        let hashed_data = keccak256(&encoded_data);
        println!("Hashed data: {}", hex::encode(&hashed_data));
    }

    #[test]
    fn sign() {
        let data = vec![
            Token::Address(*TOKEN_ADDRESS),
            Token::Uint(*AMOUNT),
            Token::Address(*RECEIVER),
            Token::FixedBytes((*TX_HASH.clone()).to_vec()),
            Token::Uint(*NONCE),
            Token::Uint(*CHAID_ID),
            Token::Bool(true)
        ];
        
        let encoded_data = encode_packed(&data).unwrap();

        println!("Encoded data: {}", hex::encode(&encoded_data));

        let hashed_data = keccak256(&encoded_data);

        println!("Hashed data: {}", hex::encode(&hashed_data));

        let secret_key = SecretKey::from_slice(&hex::decode(ECDSA_HEX_PRIVATE_KEY).unwrap()).unwrap();

        let secp = Secp256k1::new();
        
        let hashed_data = hash_message(&hashed_data);
        let eth_data = hashed_data.as_bytes();

        let msg = Message::from_slice(eth_data)
            .expect("32 bytes");
        
        println!("Msg: {}", hex::encode(eth_data));

        let sig = secp.sign_ecdsa(&msg, &secret_key);

        let mut signature = sig.serialize_compact().to_vec();

        signature.push(28);
        
        println!("Signature: {}", hex::encode(&signature));

        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        assert!(secp.verify_ecdsa(&msg, &sig, &public_key).is_ok());
    }

    #[test]
    fn recover() {
        let data = vec![
            Token::Address(*TOKEN_ADDRESS),
            Token::Uint(*AMOUNT),
            Token::Address(*RECEIVER),
            Token::FixedBytes((*TX_HASH.clone()).to_vec()),
            Token::Uint(*NONCE),
            Token::Uint(*CHAID_ID),
            Token::Bool(true)
        ];
        
        let encoded_data = encode_packed(&data).unwrap();

        let hashed_data = keccak256(&encoded_data);

        let secret_key = SecretKey::from_slice(&hex::decode(ECDSA_HEX_PRIVATE_KEY).unwrap()).unwrap();

        let secp = Secp256k1::new();
        
        let hashed_data = hash_message(&hashed_data);
        let eth_data = hashed_data.as_bytes();

        let msg = Message::from_slice(&eth_data)
            .expect("32 bytes");
        
        println!("Msg: {}", hex::encode(eth_data));

        let sig = secp.sign_ecdsa(&msg, &secret_key);

        let mut signature = sig.serialize_compact().to_vec();

        let public_key = ECDSA_HEX_PUBLIC_KEY.clone();

        let v = match get_eth_v(&signature, eth_data, &public_key) {
            Ok(v) => v,
            Err(e) => panic!("Error: {}", e)
        };
            
        signature.push(v);

       println!("Signature: {}", hex::encode(&signature));
    }
}
