use ic_cdk::{post_upgrade, pre_upgrade};

use crate::{
    types::{daemons::DaemonsStorage, Storage},
    STORAGE,
};

#[pre_upgrade]
fn pre_upgrade() {
    let storage = STORAGE.with(|s| s.take());

    ic_cdk::storage::stable_save((storage,)).expect("Failed to save storage before upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    let (mut storage,): (Storage,) =
        ic_cdk::storage::stable_restore().expect("Failed to restore storage after upgrade");

    storage.signer_job.stop();
    storage.writer_job.stop();
    storage.checker_job.stop();

    storage.signer_job.run();
    storage.writer_job.run();
    storage.checker_job.run();

    STORAGE.with(|s| s.replace(storage));

    DaemonsStorage::start_active_daemons();
}
