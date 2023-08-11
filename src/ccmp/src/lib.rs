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
use ic_cdk_timers::set_timer;

use types::{config::Config, Storage};

use crate::types::job::{Job, JobType};

const POST_INIT_PASK_DELAY: u64 = 5;

thread_local! {
    static STORAGE: RefCell<Storage> = RefCell::default();
}

#[init]
fn init(config: Config) {
    storage_set!(key, config.key);

    let mut signer_job = Job::new(config.signer_interval_secs, JobType::Signer);
    let mut writer_job = Job::new(config.writer_interval_secs, JobType::Writer);
    let mut checker_job = Job::new(config.checker_interval_secs, JobType::Checker);

    signer_job.run();
    writer_job.run();
    checker_job.run();

    storage_set!(signer_job, signer_job);
    storage_set!(writer_job, writer_job);
    storage_set!(checker_job, checker_job);

    set_timer(
        Duration::from_secs(POST_INIT_PASK_DELAY),
        tasks::post_init::execute,
    );
}

#[allow(dead_code)]
fn export_candid() -> String {
    use methods::daemons::RegisterDaemonArgs;
    use std::collections::HashMap;
    use types::{balances::Balance, chains::ChainMetadata, config::ConfigUpdate, daemons::Daemon};

    export_service!();
    __export_service()
}

// this hack is used to export candid interfaces to a candid file
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
