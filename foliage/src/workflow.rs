use crate::elm::Elm;
use bevy_ecs::system::NonSend;
use gloo_worker::{HandlerId, Worker, WorkerScope};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use winit::event_loop::EventLoopProxy;
#[trait_variant::make(Workflow: Send)]
pub trait SingleThreadedWorkflow {
    type Action: Debug + Clone + Send + Sync + Sized + 'static + Serialize + for<'a> Deserialize<'a>;
    type Response: Debug
        + Clone
        + Send
        + Sync
        + Sized
        + 'static
        + Serialize
        + for<'a> Deserialize<'a>;
    async fn process(arc: EngenHandle<Self>, action: Self::Action) -> Self::Response;
    fn react(elm: &mut Elm, response: Self::Response);
}
#[cfg(target_family = "wasm")]
pub type EngenHandle<W> = Arc<std::sync::Mutex<W>>;
#[cfg(not(target_family = "wasm"))]
pub type EngenHandle<W> = Arc<tokio::sync::Mutex<W>>;
pub type WorkflowConnection<W> = NonSend<'static, WorkflowConnectionBase<W>>;
pub struct WorkflowConnectionBase<W: Workflow + Default + Send + Sync + 'static> {
    // channel for native
    #[cfg(not(target_family = "wasm"))]
    bridge: tokio::sync::mpsc::UnboundedSender<W::Action>,
    // worker-bridge for web
    #[cfg(target_family = "wasm")]
    bridge: gloo_worker::WorkerBridge<Engen<W>>,
}
#[cfg(not(target_family = "wasm"))]
async fn native_handler<W: Workflow + Default + Send + Sync + 'static>(
    proxy: EventLoopProxy<W::Response>,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<W::Action>,
) {
    let engen = Engen(Arc::new(tokio::sync::Mutex::new(W::default())));
    loop {
        while let Some(action) = receiver.recv().await {
            let response = W::process(engen.0.clone(), action).await;
            proxy.send_event(response).unwrap()
        }
    }
}
impl<W: Workflow + Default + Send + Sync + 'static> WorkflowConnectionBase<W> {
    pub(crate) fn new(proxy: EventLoopProxy<W::Response>, _wp: String) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                use gloo_worker::Spawnable;
                let bridge = Engen::<W>::spawner()
                .callback(move |response| {
                    let _ = proxy.send_event(response);
                })
                .spawn(_wp.as_str());
            } else {
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
                tokio::task::spawn(native_handler::<W>(proxy, receiver));
                let bridge = sender;
            }
        }
        Self { bridge }
    }
    pub fn send(&self, action: W::Action) {
        #[cfg(not(target_family = "wasm"))]
        self.bridge.send(action).expect("sending-action-failed");
        #[cfg(target_family = "wasm")]
        self.bridge.send(action);
    }
}
pub(crate) struct Engen<W: Workflow + Default + Send + Sync + 'static>(pub(crate) EngenHandle<W>);
impl<W: Workflow + Default + 'static + Send + Sync> Worker for Engen<W> {
    type Message = OutputWrapper<W>;
    type Input = W::Action;
    type Output = W::Response;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "wasm")] {
                Engen(Arc::new(std::sync::Mutex::new(W::default())))
            } else {
                Engen(Arc::new(tokio::sync::Mutex::new(W::default())))
            }
        }
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        scope.respond(msg.handler_id, msg.response)
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        let arc = self.0.clone();
        scope.send_future(async move {
            let response = <W as Workflow>::process(arc, msg).await;
            OutputWrapper::new(id, response)
        })
    }
}
pub(crate) struct OutputWrapper<W: Workflow + Default + 'static + Send + Sync> {
    pub(crate) handler_id: HandlerId,
    pub(crate) response: <Engen<W> as Worker>::Output,
}

impl<T: Workflow + Default + 'static + Send + Sync> OutputWrapper<T>
where
    Self: Sized,
{
    pub(crate) fn new(handler_id: HandlerId, response: <Engen<T> as Worker>::Output) -> Self {
        Self {
            handler_id,
            response,
        }
    }
}
pub fn start_web_worker<W: Workflow + Default + 'static + Send + Sync>() {
    #[cfg(target_family = "wasm")]
    {
        use gloo_worker::Registrable;
        console_error_panic_hook::set_once();
        Engen::<W>::registrar().register();
    }
}
