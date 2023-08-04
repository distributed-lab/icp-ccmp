use futures::future::join_all;
use thiserror::Error;

use crate::{
    log, storage_get,
    types::{
        chains::{Chain, ChainType},
        evm_chains::{EvmChain, EvmChainError},
    },
};

#[derive(Error, Debug)]
pub enum ListenerError {
    #[error("unknown chain type")]
    UnknownChainType,
    #[error("evm chain error: {0}")]
    EvmChain(#[from] EvmChainError),
}

pub fn run() {
    log!("[LISTENER] starting]");
    ic_cdk::spawn(async {
        if let Err(err) = listen().await {
            log!("[LISTENER] error: {}", err);
        };
    })
}

async fn listen() -> Result<(), ListenerError> {
    let chains_storage = storage_get!(chains_storage);

    let mut futures = vec![];
    for (id, chain_metadata) in chains_storage.chains_metadata.iter() {
        match chain_metadata.chain_type {
            ChainType::Evm => {
                futures.push(EvmChain::listen(*id));
            }
            _ => return Err(ListenerError::UnknownChainType),
        }
    }

    if let Err(err) = join_all(futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        log!("[LISTENER] error: {}", err);
    };

    log!("[LISTENER] finished]");

    Ok(())
}
