use crate::asset::AssetKey;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, Request, RequestInit, RequestMode, Response};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum SystemMessageAction {
    WasmAsset(AssetKey, String),
    NoOp,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum SystemMessageResponse {
    WasmAsset(AssetKey, Vec<u8>),
    NoOp,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct ResponseMessage<R>(pub Option<R>, pub(crate) Option<SystemMessageResponse>);
impl<R> ResponseMessage<R> {
    pub(crate) fn user(r: R) -> Self {
        Self(Some(r), None)
    }
    pub(crate) fn system(sm: SystemMessageResponse) -> Self {
        Self(None, Some(sm))
    }
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct ActionMessage<A>(pub Option<A>, pub(crate) Option<SystemMessageAction>);
impl<A> ActionMessage<A> {
    pub(crate) fn user(a: A) -> Self {
        Self(Some(a), None)
    }
    pub(crate) fn system(sm: SystemMessageAction) -> Self {
        Self(None, Some(sm))
    }
}
pub(crate) async fn system_message_response(a: SystemMessageAction) -> SystemMessageResponse {
    match a {
        SystemMessageAction::WasmAsset(asset_key, path) => {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);
            let window = web_sys::window().unwrap();
            let origin = window.origin();
            let url = format!("https://{}{}", origin, path);
            if let Ok(request) = Request::new_with_str_and_init(&url, &opts) {
                if request
                    .headers()
                    .set("Accept", "application/octet-stream")
                    .is_ok()
                {
                    if let Ok(response) = JsFuture::from(window.fetch_with_request(&request)).await
                    {
                        let response: Response = response.dyn_into().unwrap();
                        if let Ok(array) = response.array_buffer() {
                            if let Ok(response) = JsFuture::from(array).await {
                                SystemMessageResponse::WasmAsset(
                                    asset_key,
                                    js_sys::Uint8Array::new(&response).to_vec(),
                                )
                            }
                        }
                    }
                }
            }
            SystemMessageResponse::NoOp
        }
        SystemMessageAction::NoOp => SystemMessageResponse::NoOp,
    }
}
#[macro_export]
macro_rules! load_asset {
    ($elm:ident, $id:expr, $p:literal, $rel:literal) => {
        #[cfg(target_family = "wasm")]
        $elm.load_web_asset($id, concat!($rel, $p));
        #[cfg(not(target_family = "wasm"))]
        $elm.load_native_asset($id, include_bytes!(concat!($rel, $p)));
    };
}
