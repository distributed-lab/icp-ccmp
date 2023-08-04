use candid::candid_method;
use ic_cdk::{api::is_controller, query, update};
use thiserror::Error;

use crate::{
    log,
    types::config::{Config, ConfigUpdate},
};

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("caller is not a controller")]
    CallerIsNotAController,
}

#[candid_method(update)]
#[update]
fn update_config(config: ConfigUpdate) -> Result<(), String> {
    _update_config(config).map_err(|e| e.to_string())
}

fn _update_config(config: ConfigUpdate) -> Result<(), ControllerError> {
    if !is_controller(&ic_cdk::caller()) {
        return Err(ControllerError::CallerIsNotAController);
    }

    config.apply();

    log!("[CONTROLLERS] config updated: {:?}", config);

    Ok(())
}

#[candid_method(query)]
#[query]
fn get_config() -> Result<Config, String> {
    _get_config().map_err(|e| e.to_string())
}

fn _get_config() -> Result<Config, ControllerError> {
    if !is_controller(&ic_cdk::caller()) {
        return Err(ControllerError::CallerIsNotAController);
    }

    Ok(Config::get())
}
