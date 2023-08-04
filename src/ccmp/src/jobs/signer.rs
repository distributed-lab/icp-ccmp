use futures::future::join_all;
use thiserror::Error;

use crate::{log, types::messages::MessageError, STORAGE};

const BATCH_TO_SIGN_SIZE: usize = 10;

#[derive(Error, Debug)]
pub enum SignerError {
    #[error("message error: {0}")]
    Message(#[from] MessageError),
}

pub fn run() {
    log!("[SIGNER] starting]");
    ic_cdk::spawn(async {
        if let Err(err) = sign().await {
            log!("[SIGNER] error: {}", err);
        };
    })
}

async fn sign() -> Result<(), SignerError> {
    let messages = STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();

        let drain_end = BATCH_TO_SIGN_SIZE.min(storage.listened_messages.len());

        storage
            .listened_messages
            .drain(..drain_end)
            .collect::<Vec<_>>()
    });

    if messages.is_empty() {
        log!("[SIGNER] finished, no messages to sign");
        return Ok(());
    }

    let mut futures = vec![];
    for message in messages {
        futures.push(message.sign());
    }

    let mut signed_messages = join_all(futures)
        .await
        .iter()
        .filter_map(|result| match result {
            Ok(message) => Some(message),
            Err(err) => {
                log!("[SIGNER] error: {}", err);
                None
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    let signed_messages_number = signed_messages.len();

    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.signed_messages.append(&mut signed_messages)
    });

    log!(
        "[SIGNER] finished, signed messages number: {}",
        signed_messages_number
    );

    Ok(())
}
