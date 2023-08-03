use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use ic_web3_rs::transports::ic_http_client::{CallOptions, CallOptionsBuilder};

pub fn call_options(transformer: String) -> CallOptions {
    CallOptionsBuilder::default()
        .transform(Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: transformer,
            }),
            context: vec![],
        }))
        .max_resp(None)
        .cycles(None)
        .build()
        .unwrap()
}
