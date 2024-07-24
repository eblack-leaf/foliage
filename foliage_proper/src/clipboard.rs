use bevy_ecs::component::Component;
#[cfg(not(target_family = "wasm"))]
use copypasta::ClipboardProvider;

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
            let promise = web_sys::window()
                .expect("window")
                .navigator()
                .clipboard()
                .unwrap()
                .write_text(data.as_str());
            wasm_bindgen_futures::spawn_local(async move {
                let _message = wasm_bindgen_futures::JsFuture::from(promise).await.ok();
            });
        }
        #[cfg(not(target_family = "wasm"))]
        if let Some(h) = self.handle.as_mut() {
            h.set_contents(data).expect("clipboard writing");
        }
    }
}
