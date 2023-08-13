use crate::{log, STORAGE};

const PENDING_TX_BATCH: usize = 10;

#[derive(Debug, thiserror::Error)]
pub enum CheckerError {}

pub fn run() {
    log!("[CHECKER] starting]");
    ic_cdk::spawn(async {
        if let Err(err) = check().await {
            log!("[CHECKER] error: {}", err);
        };
    })
}

pub async fn check() -> Result<(), CheckerError> {
    let pending_txs = STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();

        let drain_end = PENDING_TX_BATCH.min(storage.pending_txs_storage.0.len());

        storage
            .pending_txs_storage
            .0
            .drain(..drain_end)
            .collect::<Vec<_>>()
    });

    if pending_txs.is_empty() {
        log!("[CHECKER] finished, no pending txs to check");
        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.checker_job.stop();
        });
        return Ok(());
    }

    let mut futures = vec![];
    for pending_tx in pending_txs.iter() {
        futures.push(pending_tx.clone().check());
    }

    let results = futures::future::join_all(futures).await;

    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();

        for (i, result) in results.into_iter().enumerate() {
            let pending_tx = &pending_txs[i];

            if let Err(err) = result {
                log!("[CHECKER] error: {}", err);
                continue;
            }

            match result {
                Ok(is_finished) => {
                    if !is_finished {
                        storage.pending_txs_storage.0.push(pending_tx.clone());
                    }
                }
                Err(err) => {
                    log!("[CHECKER] error: {}", err);
                    storage.pending_txs_storage.0.push(pending_tx.clone())
                }
            }
        }
    });

    log!(
        "[CHECKER] finished, pending txs checked: {}",
        pending_txs.len()
    );

    Ok(())
}
