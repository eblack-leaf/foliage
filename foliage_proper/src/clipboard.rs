use crate::element::{IdTable, TargetHandle};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Entity, World};
use bevy_ecs::system::{Command, Commands, Query};
#[cfg(not(target_family = "wasm"))]
use copypasta::ClipboardProvider;
use futures_channel::oneshot;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Object;
use web_sys::wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone)]
pub struct ClipboardWrite {
    message: String,
}
impl ClipboardWrite {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self {
            message: s.as_ref().to_string(),
        }
    }
}
impl Command for ClipboardWrite {
    fn apply(self, world: &mut World) {
        world
            .get_non_send_resource_mut::<Clipboard>()
            .unwrap()
            .write(self.message);
    }
}
pub type ClipboardReadFn<B> = fn(String) -> B;
#[derive(Clone)]
pub struct ClipboardRead<B: Bundle> {
    target: TargetHandle,
    on_read: Box<ClipboardReadFn<B>>,
}
impl<B: Bundle> ClipboardRead<B> {
    pub fn new(target: TargetHandle, on_read: ClipboardReadFn<B>) -> Self {
        Self {
            target,
            on_read: Box::new(on_read),
        }
    }
}
#[derive(Component)]
pub(crate) struct ClipboardReadRetrieve<B: Bundle> {
    message_recv: oneshot::Receiver<String>,
    on_read: Box<ClipboardReadFn<B>>,
}
pub(crate) fn read_retrieve<B: Bundle + Send + Sync + 'static>(
    mut texts: Query<(Entity, &mut ClipboardReadRetrieve<B>)>,
    mut cmd: Commands,
) {
    for (entity, mut retrieve) in texts.iter_mut() {
        if let Some(m) = retrieve.message_recv.try_recv().ok() {
            if let Some(m) = m {
                cmd.entity(entity).insert((retrieve.on_read)(m));
            }
        }
    }
}
impl<B: Bundle> Command for ClipboardRead<B> {
    fn apply(self, world: &mut World) {
        let entity = world
            .get_resource::<IdTable>()
            .unwrap()
            .lookup_target(self.target)
            .unwrap();
        #[cfg(not(target_family = "wasm"))]
        {
            let message = world
                .get_non_send_resource_mut::<Clipboard>()
                .unwrap()
                .read();
            world.entity_mut(entity).insert((self.on_read)(message));
        }
        #[cfg(target_family = "wasm")]
        {
            let (sender, recv) = oneshot::channel();
            // spawn local w/ sender
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(c) = web_sys::window().expect("window").navigator().clipboard() {
                    let _message = wasm_bindgen_futures::JsFuture::from(c.read_text())
                        .await
                        .ok();
                    if let Some(m) = _message {
                        sender.send(m.as_string().unwrap()).ok();
                    }
                }
            });
            world.entity_mut(entity).insert(ClipboardReadRetrieve {
                message_recv: recv,
                on_read: self.on_read,
            });
        }
    }
}
pub struct Clipboard {
    #[cfg(not(target_family = "wasm"))]
    pub handle: Option<copypasta::ClipboardContext>,
    #[cfg(target_family = "wasm")]
    pub handle: Option<()>,
}
impl Clipboard {
    #[cfg(not(target_family = "wasm"))]
    pub(crate) fn new() -> Self {
        let handle = copypasta::ClipboardContext::new();
        Self {
            handle: if handle.is_ok() {
                Some(handle.expect("clipboard"))
            } else {
                None
            },
        }
    }
    #[cfg(target_family = "wasm")]
    pub(crate) fn new() -> Self {
        let handle = web_sys::window().expect("window").navigator().clipboard();
        if handle.is_some() {
            let document = web_sys::window().unwrap().document().unwrap();
            let node = document.create_element("div").unwrap();
            node.set_inner_html("<div id='copy-trigger'></div>");
            document.body().unwrap().append_child(&node).unwrap();
        }
        Self {
            handle: if handle.is_some() { Some(()) } else { None },
        }
    }
    #[cfg(not(target_family = "wasm"))]
    pub fn read(&mut self) -> String {
        if self.handle.is_none() {
            return String::default();
        }
        return self
            .handle
            .as_mut()
            .unwrap()
            .get_contents()
            .unwrap_or_default();
    }
    pub fn write(&mut self, data: String) {
        if self.handle.is_none() {
            return;
        }
        #[cfg(target_family = "wasm")]
        {
            if web_sys::window()
                .expect("window")
                .navigator()
                .clipboard()
                .is_some()
            {
                // current working method
                let promise = web_sys::window()
                    .expect("window")
                    .navigator()
                    .clipboard()
                    .unwrap()
                    .write_text(data.as_str());
                wasm_bindgen_futures::spawn_local(async move {
                    let _message = wasm_bindgen_futures::JsFuture::from(promise).await.ok();
                });

                // TODO rework below
                // let node = web_sys::window()
                //     .unwrap()
                //     .document()
                //     .unwrap()
                //     .get_element_by_id("copy-trigger")
                //     .unwrap()
                //     .dyn_into::<web_sys::HtmlElement>()
                //     .unwrap();
                // let closure = wasm_bindgen::prelude::Closure::once(move || {
                //     tracing::trace!("writing clipboard {:?}", data);
                //     let js_string = JsValue::from_str(data.as_str());
                //     let js_array = web_sys::js_sys::Array::from_iter(std::iter::once(js_string));
                //     tracing::trace!("js-array {:?}", js_array);
                //     let js_blob = web_sys::Blob::new_with_str_sequence_and_options(
                //         &js_array,
                //         &BlobPropertyBag::new().type_("text/plain"),
                //     )
                //     .unwrap();
                //     let inner_promise = wasm_bindgen_futures::js_sys::Promise::resolve(&js_blob);
                //     let js_obj = Object::new();
                //     web_sys::js_sys::Reflect::set(&js_obj, &"text/plain".into(), &inner_promise)
                //         .unwrap();
                //     let item = ClipboardItemExt::new(&js_obj, &Object::new());
                //     let item_array = web_sys::js_sys::Array::from(item.as_ref());
                //
                //     wasm_bindgen_futures::spawn_local(async move {
                //         let promise = web_sys::window()
                //             .expect("window")
                //             .navigator()
                //             .clipboard()
                //             .unwrap()
                //             .write(&item_array);
                //         let _message = wasm_bindgen_futures::JsFuture::from(promise).await.ok();
                //         tracing::trace!("return message {:?}", _message);
                //     });
                // });
                // node.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                //     .unwrap();
                // node.click();
                // node.remove_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                //     .unwrap();
                // closure.forget();
            }
        }
        #[cfg(not(target_family = "wasm"))]
        if let Some(h) = self.handle.as_mut() {
            h.set_contents(data).expect("clipboard writing");
        }
    }
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = ClipboardItem, extends = web_sys::ClipboardItem)]
    type ClipboardItemExt;

    #[wasm_bindgen(constructor, js_class = ClipboardItem)]
    fn new(data: &JsValue, options: &Object) -> ClipboardItemExt;
}
