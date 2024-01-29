use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum SystemMessageAction {
    NoOp,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum SystemMessageResponse {
    NoOp,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResponseMessage<R>(pub Option<R>, pub(crate) Option<SystemMessageResponse>);
impl<R> ResponseMessage<R> {
    pub(crate) fn user(r: R) -> Self {
        Self(Some(r), None)
    }
    pub(crate) fn system(sm: SystemMessageResponse) -> Self {
        Self(None, Some(sm))
    }
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ActionMessage<A>(pub Option<A>, pub(crate) Option<SystemMessageAction>);
impl<A> ActionMessage<A> {
    pub(crate) fn user(a: A) -> Self {
        Self(Some(a), None)
    }
    pub(crate) fn system(sm: SystemMessageAction) -> Self {
        Self(None, Some(sm))
    }
}
pub(crate) async fn system_message_response(a: SystemMessageAction) -> SystemMessageResponse {
    SystemMessageResponse::NoOp
}
