use futures::future::join_all;
use thiserror::Error;

use crate::{log, STORAGE};

const BATCH_TO_WRITE_SIZE: usize = 10;

#[derive(Error, Debug)]
pub enum WriterError {}

pub fn run() {
    log!("[WRITER] starting]");
    ic_cdk::spawn(async {
        if let Err(err) = write().await {
            log!("[WRITER] error: {}", err);
        };
    })
}

async fn write() -> Result<(), WriterError> {
    let messages = STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();

        let drain_end = BATCH_TO_WRITE_SIZE.min(storage.signed_messages.len());

        storage
            .signed_messages
            .drain(..drain_end)
            .collect::<Vec<_>>()
    });

    if messages.is_empty() {
        log!("[WRITER] finished, no messages to write");
        return Ok(());
    }

    let mut futures = vec![];
    for message in messages {
        futures.push(message.send());
    }

    if let Err(err) = join_all(futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        log!("[LISTENER] WRITER: {}", err);
    };

    log!("[WRITER] finished]");
    Ok(())
}
