mod jobs;
mod macros;
mod methods;
mod migrations;
mod tasks;
mod types;
mod utils;

use std::{cell::RefCell, time::Duration};

use candid::export_service;
use ic_cdk::init;
use ic_cdk_timers::{set_timer, set_timer_interval};

use jobs::{listener, signer, writer};
use types::{config::Config, Storage};

const POST_INIT_PASK_DELAY: u64 = 5;

thread_local! {
    static STORAGE: RefCell<Storage> = RefCell::default();
}

#[init]
fn init(config: Config) {
    config.apply();

    set_timer_interval(
        Duration::from_secs(config.listener_interval_secs),
        listener::run,
    );

    set_timer_interval(
        Duration::from_secs(config.signer_interval_secs),
        signer::run,
    );

    set_timer_interval(
        Duration::from_secs(config.writer_interval_secs),
        writer::run,
    );

    set_timer(
        Duration::from_secs(POST_INIT_PASK_DELAY),
        tasks::post_init::execute,
    );
}

#[allow(dead_code)]
fn export_candid() -> String {
    use std::collections::HashMap;
    use types::{chains::ChainMetadata, config::ConfigUpdate};

    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::export_candid;

    #[test]
    fn save_candid() {
        let dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("src")
            .join("ccmp");

        std::fs::write(dir.join("ccmp.did"), export_candid()).expect("Write failed.");
    }
}
