use crate::elm::Elm;
use gloo_worker::{HandlerId, Worker, WorkerScope};
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoopProxy;

pub trait Workflow {
    type Action;
    type Response;
    async fn process(arc: Arc<Mutex<Self>>, action: Self::Action) -> Self::Response;
    fn react(elm: &mut Elm, response: Self::Response);
}
pub(crate) struct WorkflowConnection<W: Workflow + Default> {
    // channel for native
    #[cfg(not(target_family = "wasm"))]
    bridge: tokio::sync::mpsc::UnboundedSender<W::Action>,
    // worker-bridge for web
    #[cfg(target_family = "wasm")]
    bridge: gloo_worker::WorkerBridge<Engen<W>>,
}
impl<W: Workflow + Default> WorkflowConnection<W> {
    pub(crate) fn new(proxy: EventLoopProxy<W::Response>, _wp: String) -> Self {
        Self {
            bridge: {
                let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
                tokio::task::spawn(async move {
                    let engen = Engen(Arc::new(Mutex::new(W::default())));
                    loop {
                        while let Some(action) = receiver.recv().await {
                            let response = W::process(engen.0.clone(), action).await;
                            proxy.send_event(response).expect("sending-response-failed")
                        }
                    }
                });
                sender
            },
            #[cfg(target_family = "wasm")]
            bridge: Engen::<W>::spawner()
                .callback(move |response| {
                    let _ = proxy.send_event(response);
                })
                .spawn(_wp.as_str()),
        }
    }
    pub fn send(&self, action: W::Action) {
        #[cfg(not(target_family = "wasm"))]
        self.bridge.send(action).expect("sending-action-failed");
        #[cfg(target_family = "wasm")]
        self.bridge.send(action);
    }
}
pub(crate) struct Engen<W: Workflow + Default>(pub(crate) Arc<Mutex<W>>);
impl<W: Workflow + Default> Worker for Engen<W> {
    type Message = OutputWrapper<W>;
    type Input = W::Action;
    type Output = W::Response;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Engen(Arc::new(Mutex::new(W::default())))
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
pub(crate) struct OutputWrapper<W: Workflow + Default + 'static> {
    pub(crate) handler_id: HandlerId,
    pub(crate) response: <Engen<W> as Worker>::Output,
}

impl<T: Workflow + Default + 'static> OutputWrapper<T>
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
pub fn start_web_worker<W: Workflow + Default + 'static>() {
    #[cfg(target_family = "wasm")]
    {
        use gloo_worker::Registrable;
        console_error_panic_hook::set_once();
        Engen::<W>::registrar().register();
    }
}