use crate::elm::Elm;
use crate::system_message::{
    system_message_response, ActionMessage, ResponseMessage, SystemMessageAction,
    SystemMessageResponse,
};
use gloo_worker::{HandlerId, Worker, WorkerScope};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use winit::event_loop::EventLoopProxy;
pub trait Workflow
where
    Self: Send + Sync + 'static + Sized,
{
    type Action: Debug + Clone + Send + Sync + Sized + 'static + Serialize + for<'a> Deserialize<'a>;
    type Response: Debug
        + Clone
        + Send
        + Sync
        + Sized
        + 'static
        + Serialize
        + for<'a> Deserialize<'a>;
    fn workflow<Fut: Future<Output = Self::Response>>() -> Box<fn(action: Self::Action) -> Fut>;
    fn react(elm: &mut Elm, response: Self::Response);
}
pub struct WorkflowHandle<W: Workflow> {
    // channel for native
    #[cfg(not(target_family = "wasm"))]
    bridge: tokio::sync::mpsc::UnboundedSender<ActionMessage<W::Action>>,
    // worker-bridge for web
    #[cfg(target_family = "wasm")]
    bridge: gloo_worker::WorkerBridge<WorkflowEngen<W>>,
}
#[cfg(not(target_family = "wasm"))]
async fn native_handler<W: Workflow, Fut: Future<Output = W::Response>>(
    proxy: EventLoopProxy<ResponseMessage<W::Response>>,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<ActionMessage<W::Action>>,
    fut: fn(action: W::Action) -> Fut,
) {
    loop {
        while let Some(action) = receiver.recv().await {
            if let Some(a) = action.0 {
                let response = fut(a).await;
                proxy.send_event(ResponseMessage::user(response)).unwrap();
            } else if let Some(s) = action.1 {
                let response = system_message_response(s).await;
                proxy.send_event(ResponseMessage::system(response)).unwrap()
            }
        }
    }
}
impl<W: Workflow> WorkflowHandle<W> {
    pub(crate) fn new<Fut: Future<Output = W::Response> + std::marker::Send + 'static>(
        proxy: EventLoopProxy<ResponseMessage<W::Response>>,
        _wp: String,
        fut: fn(action: W::Action) -> Fut,
    ) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                use gloo_worker::Spawnable;
                let bridge = WorkflowEngen::<W>::spawner()
                .callback(move |response| {
                    let _ = proxy.send_event(response);
                })
                .spawn(_wp.as_str());
            } else {
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
                tokio::task::spawn(native_handler::<W, Fut>(proxy, receiver, fut));
                let bridge = sender;
            }
        }
        Self { bridge }
    }
    pub fn send(&self, action: W::Action) {
        #[cfg(not(target_family = "wasm"))]
        self.bridge
            .send(ActionMessage::user(action))
            .expect("sending-action-failed");
        #[cfg(target_family = "wasm")]
        self.bridge.send(ActionMessage::user(action));
    }
    #[allow(unused)]
    pub(crate) fn system_send(&self, system_message: SystemMessageAction) {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                self.bridge.send(ActionMessage::system(system_message));
            } else {
                self.bridge.send(ActionMessage::system(system_message)).expect("sending-action-failed")
            }
        }
    }
}
struct WorkflowEngen<W: Workflow, Fut>
where
    Fut: Future<Output = W::Response> + std::clone::Clone,
{
    pub process: fn(action: W::Action) -> Fut,
}
impl<W: Workflow, Fut> Worker for WorkflowEngen<W, Fut>
where
    Fut: Future<Output = W::Response> + 'static + std::clone::Clone,
{
    type Message = OutputWrapper<W, Fut>;
    type Input = ActionMessage<W::Action>;
    type Output = ResponseMessage<W::Response>;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        let process = W::workflow::<Fut>();
        WorkflowEngen::<W, Fut> { process: *process }
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        scope.respond(msg.handler_id, msg.response)
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        let func = self.process;
        scope.send_future(async move {
            let response = if let Some(a) = msg.0 {
                ResponseMessage::user(func(a).await)
            } else if let Some(s) = msg.1 {
                ResponseMessage::system(system_message_response(s).await)
            } else {
                ResponseMessage::system(SystemMessageResponse::NoOp)
            };
            OutputWrapper::new(id, response)
        });
    }
}
pub(crate) struct OutputWrapper<
    W: Workflow,
    Fut: Future<Output = W::Response> + 'static + std::clone::Clone,
> {
    pub(crate) handler_id: HandlerId,
    pub(crate) response: <WorkflowEngen<W, Fut> as Worker>::Output,
}

impl<W: Workflow, Fut: Future<Output = W::Response> + std::clone::Clone> OutputWrapper<W, Fut>
where
    Self: Sized,
{
    pub(crate) fn new(
        handler_id: HandlerId,
        response: <WorkflowEngen<W, Fut> as Worker>::Output,
    ) -> Self {
        Self {
            handler_id,
            response,
        }
    }
}
pub fn start_web_worker<W: Workflow>() {
    #[cfg(target_family = "wasm")]
    {
        use gloo_worker::Registrable;
        console_error_panic_hook::set_once();
        Engen::<W>::registrar().register();
    }
}