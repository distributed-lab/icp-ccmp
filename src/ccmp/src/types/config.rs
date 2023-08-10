use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{storage_get, storage_set, STORAGE};

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    pub key: String,
    pub signer_interval_secs: u64,
    pub writer_interval_secs: u64,
}

impl Config {
    pub fn get() -> Config {
        Config {
            key: storage_get!(key),
            signer_interval_secs: storage_get!(signer_job).interval_secs,
            writer_interval_secs: storage_get!(writer_job).interval_secs,
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct ConfigUpdate {
    key: Option<String>,
    signer_interval_secs: Option<u64>,
    writer_interval_secs: Option<u64>,
}

impl ConfigUpdate {
    pub fn apply(&self) {
        if let Some(key) = &self.key {
            storage_set!(key, key.clone());
        }

        STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();

            if let Some(signer_interval_secs) = &self.signer_interval_secs {
                storage
                    .signer_job
                    .update_interval_secs(*signer_interval_secs);
            }

            if let Some(writer_interval_secs) = &self.writer_interval_secs {
                storage
                    .writer_job
                    .update_interval_secs(*writer_interval_secs);
            }
        });
    }
}
