mod balances;
mod chains;
mod controllers;
pub mod daemons;
mod transforms;

use candid::candid_method;
use ic_cdk::{query, update};
use ic_utils::{
    api_type::{GetInformationRequest, GetInformationResponse, UpdateInformationRequest},
    get_information, update_information,
};

use crate::types::Storage;

#[candid_method(update)]
#[update]
async fn get_public_key() -> Result<String, String> {
    Storage::get_public_key().await.map_err(|e| e.to_string())
}

#[query(name = "getCanistergeekInformation")]
pub async fn get_canistergeek_information(
    request: GetInformationRequest,
) -> GetInformationResponse<'static> {
    get_information(request)
}

#[update(name = "updateCanistergeekInformation")]
pub async fn update_canistergeek_information(request: UpdateInformationRequest) {
    update_information(request);
}
