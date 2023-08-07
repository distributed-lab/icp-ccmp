use futures::future::join_all;
use itertools::Itertools;
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
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.writer_job.stop();
        });
        return Ok(());
    }

    let futures = messages
        .into_iter()
        .group_by(|msg| msg.to_chain_id.clone())
        .into_iter()
        .map(|(_, group)| {
            let group = group.collect::<Vec<_>>();
            async move {
                for message in group {
                    if let Err(err) = message.send().await {
                        log!("[WRITER]: error {}", err);
                    };
                }
            }
        })
        .collect::<Vec<_>>();

    join_all(futures).await;

    log!("[WRITER] finished]");
    Ok(())
}
