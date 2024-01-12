use crate::{Attach, Visualizer};
#[cfg(not(target_family = "wasm"))]
use copypasta::ClipboardProvider;
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
    pub fn write(&mut self, data: String) {
        if self.handle.is_none() {
            return;
        }
        #[cfg(target_family = "wasm")]
        if let Some(h) = web_sys::window().expect("window").navigator().clipboard() {
            // TODO handle promise
            let _promise = h.write_text(data.as_str());
        }
        #[cfg(not(target_family = "wasm"))]
        if let Some(h) = self.handle.as_mut() {
            h.set_contents(data).expect("clipboard writing");
        }
    }
}
pub(crate) struct ClipboardAttachment;
impl Attach for ClipboardAttachment {
    fn attach(visualizer: &mut Visualizer) {
        visualizer
            .job
            .container
            .insert_non_send_resource(Clipboard::new());
    }
}
