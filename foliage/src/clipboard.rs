//! The clipboard -- what is put on it, and what comes back off it.
//!
//! A write is answered where it is made. A **read is not**: on the web it is a promise, and on a
//! desktop it is a round trip to whichever program owns the selection. So a read is a request, and
//! what it finds is an **op** -- pushed onto the one queue from wherever it finished, ordered by
//! when it arrived like every other change, and drained where every other change is drained. That is
//! the road [`asset`](crate::asset) already takes, and taking it here is what makes a paste mean the
//! same thing on both targets rather than landing in this frame on one and the next on the other.
//!
//! The platform's own clipboard is opened by `photosynthesize` and left shut under the headless
//! suite -- the seam [`Wake`] already sits on. A test therefore reads and writes the engine's own
//! mirror and never touches the clipboard of whoever is running it.

use tracing::debug;

use crate::leaf::Leaf;
use crate::op::Op;
use crate::queue::{Queue, Wake};

/// The system clipboard, and what this program last put on it.
///
/// The mirror is not a cache. It is the answer where the platform has none to give -- a desktop with
/// no display server, a browser that refuses to be read without a gesture it did not get -- so a
/// copy inside an app still round-trips to a paste inside it, whatever the host allows.
#[derive(Default)]
pub(crate) struct Clipboard {
    /// What this program last wrote. Read back when the platform will not answer.
    mirror: String,
    /// The platform's own. Absent until [`attach`](Clipboard::attach), and absent for good under the
    /// headless suite.
    system: Option<System>,
}

/// The platform's own clipboard, once it has been opened.
///
/// Native holds its context for the life of the program because on X11 the program that wrote the
/// selection is the one that serves it -- a context dropped after a write takes what it wrote with
/// it. The web holds nothing: `navigator.clipboard` is reached by name each time.
#[cfg(not(target_family = "wasm"))]
struct System(arboard::Clipboard);

#[cfg(target_family = "wasm")]
struct System;

impl Clipboard {
    /// Opens the platform's own clipboard, once, at boot.
    ///
    /// A desktop with nothing to open one against is not an error: the mirror answers, and the
    /// reason is traced once rather than at every copy.
    pub(crate) fn attach(&mut self) {
        #[cfg(not(target_family = "wasm"))]
        match arboard::Clipboard::new() {
            Ok(clipboard) => self.system = Some(System(clipboard)),
            Err(reason) => tracing::warn!(%reason, "no system clipboard"),
        }
        #[cfg(target_family = "wasm")]
        {
            self.system = Some(System);
        }
    }

    /// Puts `text` on the clipboard.
    ///
    /// The mirror is written whether or not the platform took it, because what an app copied is
    /// what it can paste back even where the host says no.
    pub(crate) fn write(&mut self, text: String) {
        match &mut self.system {
            #[cfg(not(target_family = "wasm"))]
            Some(System(clipboard)) => {
                if let Err(reason) = clipboard.set_text(&text) {
                    tracing::warn!(%reason, "clipboard write refused");
                }
            }
            #[cfg(target_family = "wasm")]
            Some(System) => {
                if let Some(window) = web_sys::window() {
                    let _ = window.navigator().clipboard().write_text(&text);
                }
            }
            None => {}
        }
        debug!(characters = text.chars().count(), "copied");
        self.mirror = text;
    }

    /// Asks the platform what is on the clipboard, for whoever wants it.
    ///
    /// `into` is the field that asked to be pasted into, or `None` where the app asked for itself.
    /// It is carried through to the arrival because the name was handed out before anything was
    /// read, exactly as an asset's [`Destination`](crate::asset::Destination) is.
    ///
    /// Whatever the platform says, an answer is always pushed. There is one thing an app or a field
    /// does about an empty clipboard and about one it was not allowed to read, so the two are one
    /// outcome and the reason is traced rather than reported.
    pub(crate) fn read(&mut self, queue: &Queue, wake: &Wake, into: Option<Leaf>) {
        debug!(into = into.map(|leaf| leaf.id()), "pasting");
        match &mut self.system {
            #[cfg(not(target_family = "wasm"))]
            Some(System(clipboard)) => {
                // On the frame's own thread, unlike a file, because the context that serves what
                // this program wrote has to outlive the write and so cannot be moved onto a thread
                // per read. A selection round trip is bounded and human-triggered; a font is
                // neither.
                let text = match clipboard.get_text() {
                    Ok(text) => text,
                    Err(reason) => {
                        debug!(%reason, "clipboard read refused");
                        self.mirror.clone()
                    }
                };
                answered(queue, wake, into, text);
            }
            #[cfg(target_family = "wasm")]
            Some(System) => {
                let (queue, wake, mirror) = (queue.clone(), wake.clone(), self.mirror.clone());
                wasm_bindgen_futures::spawn_local(async move {
                    let text = read_text().await.unwrap_or(mirror);
                    answered(&queue, &wake, into, text);
                });
            }
            // Nothing was ever attached, which is the headless suite. The mirror is the whole
            // clipboard there, and it answers at once so a test reads a paste in the next frame the
            // way a platform's would.
            None => answered(queue, wake, into, self.mirror.clone()),
        }
    }
}

/// Queues what the clipboard held and asks for the frame that will drain it.
///
/// The whole of what a read does when it finishes, so every one of them lands the same way whatever
/// answered it.
fn answered(queue: &Queue, wake: &Wake, into: Option<Leaf>, text: String) {
    queue.push(Op::Pasted { into, text });
    wake.rouse();
}

/// The read itself, as a promise.
///
/// Refused far more often than it is answered -- it is permission-gated, and outside a user gesture
/// most browsers say no. Each failure is one failure to an app, so none is distinguished.
#[cfg(target_family = "wasm")]
async fn read_text() -> Option<String> {
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window()?;
    let text = JsFuture::from(window.navigator().clipboard().read_text())
        .await
        .ok()?;
    text.as_string()
}
