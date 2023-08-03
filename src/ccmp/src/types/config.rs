use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{storage_set, storage_get};

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    pub key: String,
    pub listener_interval_secs: u64,
    pub signer_interval_secs: u64,
    pub writer_interval_secs: u64,
}

impl Config {
    pub fn apply(&self) {
        storage_set!(key, self.key.clone());
        storage_set!(listener_interval_secs, self.listener_interval_secs);
        storage_set!(signer_interval_secs, self.signer_interval_secs);
        storage_set!(writer_interval_secs, self.writer_interval_secs);
    }

    pub fn get() -> Config {
        Config {
            key: storage_get!(key),
            listener_interval_secs: storage_get!(listener_interval_secs),
            signer_interval_secs: storage_get!(signer_interval_secs),
            writer_interval_secs: storage_get!(writer_interval_secs),
        }
    }
}

#[derive(CandidType, Deserialize, Serialize, Debug, Default, Clone)]
pub struct ConfigUpdate {
    key: Option<String>,
    listener_interval: Option<u64>,
    signer_interval_secs: Option<u64>,
}

impl ConfigUpdate {
    pub fn apply(&self) {
        if let Some(key) = &self.key {
            storage_set!(key, key.clone());
        }

        if let Some(listener_interval) = &self.listener_interval {
            storage_set!(listener_interval_secs, *listener_interval);
        }

        if let Some(signer_interval_secs) = &self.signer_interval_secs {
            storage_set!(signer_interval_secs, *signer_interval_secs);
        }

        if let Some(writer_interval_secs) = &self.signer_interval_secs {
            storage_set!(writer_interval_secs, *writer_interval_secs);
        }
    }
}
