use serde::{Deserialize, Serialize};

use crate::asset::AssetKey;

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
#[allow(unused)]
pub(crate) async fn system_message_response(a: SystemMessageAction) -> SystemMessageResponse {
    match a {
        SystemMessageAction::WasmAsset(asset_key, path) => {
            #[cfg(target_family = "wasm")]
            {
                let res = reqwest::Client::new()
                    .get(path)
                    .header("Accept", "application/octet-stream")
                    .send()
                    .await
                    .unwrap();
                let bytes = res.bytes().await.unwrap();
                return SystemMessageResponse::WasmAsset(asset_key, bytes.to_vec());
            }
            SystemMessageResponse::NoOp
        }
        SystemMessageAction::NoOp => SystemMessageResponse::NoOp,
    }
}
