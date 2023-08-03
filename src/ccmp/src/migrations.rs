use std::time::Duration;

use ic_cdk::{post_upgrade, pre_upgrade};
use ic_cdk_timers::set_timer_interval;

use crate::{jobs::{listener, signer, writer}, types::Storage, STORAGE};

#[pre_upgrade]
fn pre_upgrade() {
    let storage = STORAGE.with(|s| s.take());
    ic_cdk::storage::stable_save((storage,)).expect("Failed to save storage before upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    let (storage,): (Storage,) =
        ic_cdk::storage::stable_restore().expect("Failed to restore storage after upgrade");

    set_timer_interval(
        Duration::from_secs(storage.listener_interval_secs),
        listener::run,
    );

    set_timer_interval(
        Duration::from_secs(storage.signer_interval_secs),
        signer::run,
    );

    set_timer_interval(
        Duration::from_secs(storage.writer_interval_secs),
        writer::run,
    );

    STORAGE.with(|s| *s.borrow_mut() = storage);
}
