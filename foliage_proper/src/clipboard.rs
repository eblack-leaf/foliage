use crate::{Attachment, Foliage, Resource};

/// System-clipboard access with a deterministic fallback. Native reads/writes go through
/// `copypasta`; on web, `write` is real (navigator.clipboard.writeText inside a user gesture)
/// but `read` returns only the app-internal mirror — the async, permission-gated
/// `navigator.clipboard.readText()` cannot resolve inside a synchronous observer, so pasting
/// content copied *outside* the app is out of scope on web (copy within the app round-trips
/// through `local`). Future path for external web paste: paste events on the hidden
/// virtual-keyboard inputs.
#[derive(Resource)]
pub struct Clipboard {
    local: String,
    #[cfg(not(target_family = "wasm"))]
    provider: Option<copypasta::ClipboardContext>,
}
impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}
impl Clipboard {
    /// Opens the system clipboard.
    pub fn new() -> Self {
        Self {
            local: String::new(),
            #[cfg(not(target_family = "wasm"))]
            provider: copypasta::ClipboardContext::new().ok(),
        }
    }
    /// Puts `s` on the system clipboard.
    pub fn write<S: Into<String>>(&mut self, s: S) {
        let s = s.into();
        self.local = s.clone();
        #[cfg(not(target_family = "wasm"))]
        {
            use copypasta::ClipboardProvider;
            if let Some(provider) = self.provider.as_mut() {
                provider.set_contents(s).ok();
            }
        }
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.navigator().clipboard().write_text(&s);
            }
        }
    }
    /// Reads the system clipboard, empty if it holds no text or is unavailable.
    pub fn read(&mut self) -> String {
        #[cfg(not(target_family = "wasm"))]
        {
            use copypasta::ClipboardProvider;
            if let Some(provider) = self.provider.as_mut() {
                if let Ok(contents) = provider.get_contents() {
                    self.local = contents;
                }
            }
        }
        self.local.clone()
    }
}
impl Attachment for Clipboard {
    fn attach(foliage: &mut Foliage) {
        foliage.world.insert_resource(Clipboard::new());
    }
}
