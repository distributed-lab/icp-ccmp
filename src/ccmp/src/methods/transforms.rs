use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk::query;

#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    response.response
}
