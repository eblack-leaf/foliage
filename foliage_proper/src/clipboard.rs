use bevy_ecs::component::Component;
#[cfg(not(target_family = "wasm"))]
use copypasta::ClipboardProvider;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Object;

#[derive(Clone, Component)]
pub struct ClipboardWrite {
    pub(crate) message: String,
}
impl ClipboardWrite {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self {
            message: s.as_ref().to_string(),
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
        if handle.is_some() {}
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
            // let promise = web_sys::window()
            //     .expect("window")
            //     .navigator()
            //     .clipboard()
            //     .unwrap()
            //     .write_text(data.as_str());
            // wasm_bindgen_futures::spawn_local(async move {
            //     let _message = wasm_bindgen_futures::JsFuture::from(promise).await.ok();
            // });

            tracing::trace!("writing clipboard {:?}", data);
            let js_string = JsValue::from_str(data.as_str());
            let js_array = web_sys::js_sys::Array::from_iter(std::iter::once(js_string));
            tracing::trace!("js-array {:?}", js_array);
            let js_blob = web_sys::Blob::new_with_str_sequence_and_options(
                &js_array,
                &web_sys::BlobPropertyBag::new().type_("text/plain"),
            )
            .unwrap();
            let inner_promise = wasm_bindgen_futures::js_sys::Promise::resolve(&js_blob);
            let js_obj = Object::new();
            web_sys::js_sys::Reflect::set(&js_obj, &"text/plain".into(), &inner_promise).unwrap();
            let item = ClipboardItemExt::new(&js_obj, &Object::new());
            let item_array = web_sys::js_sys::Array::of1(item.as_ref());

            wasm_bindgen_futures::spawn_local(async move {
                let promise = web_sys::window()
                    .expect("window")
                    .navigator()
                    .clipboard()
                    .unwrap()
                    .write(&item_array);
                let _message = wasm_bindgen_futures::JsFuture::from(promise).await.ok();
                tracing::trace!("return message {:?}", _message);
            });
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
