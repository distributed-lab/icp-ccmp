use thiserror::Error;

use crate::{
    log,
    types::{Storage, StorageError},
};

#[derive(Error, Debug)]
pub enum PostInitError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}

pub fn execute() {
    log!("[POST INIT] starting]");
    ic_cdk::spawn(async {
        if let Err(err) = post_init().await {
            log!("[POST INIT] error: {}", err);
        };
    })
}

async fn post_init() -> Result<(), PostInitError> {
    Storage::get_public_key().await?;
    log!("[POST INIT] finished]");
    Ok(())
}
