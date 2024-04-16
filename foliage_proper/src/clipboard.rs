#[cfg(not(target_family = "wasm"))]
use copypasta::ClipboardProvider;

use crate::elm::config::ElmConfiguration;
use crate::elm::Elm;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};

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
        Self {
            handle: if handle.is_some() { Some(()) } else { None },
        }
    }
    pub fn read(&mut self) -> String {
        if self.handle.is_none() {
            return String::default();
        }
        #[cfg(target_family = "wasm")]
        {
            // TODO "move this to system-message"? cant get value
            // TODO back from future if put in spawn_local
            // TODO also window() + navigator() not in web-worker?
            return String::default();
        }
        #[cfg(not(target_family = "wasm"))]
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
            // TODO may not work as in separate web-worker? can access clipboard to write to?
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(h) = web_sys::window().expect("window").navigator().clipboard() {
                    let _promise = h.write_text(data.as_str());
                    wasm_bindgen_futures::JsFuture::from(_promise)
                        .await
                        .expect("clipboard-error");
                    return;
                }
            });
        }
        #[cfg(not(target_family = "wasm"))]
        if let Some(h) = self.handle.as_mut() {
            h.set_contents(data).expect("clipboard writing");
        }
    }
}
impl Leaf for Clipboard {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.container().insert_non_send_resource(Clipboard::new());
    }
}