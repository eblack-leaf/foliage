//! A URL handed to the host.
//!
//! Two things an app can ask for that the engine has no way to do itself: go somewhere, and save
//! something. Both are the host's, and both are nameable on every target because where an app's
//! links point is a fact about the app rather than about what it was built for.
//!
//! Handed over by `photosynthesize` and left in the engine's hands under the headless suite -- the
//! seam [`Wake`](crate::queue::Wake) sits on. A test therefore opens no browser on whoever is
//! running it.

use tracing::debug;

/// Whether a URL reaches the host at all.
///
/// One flag rather than a handle, because neither of these holds anything: a navigation is a line
/// of JavaScript or a process the desktop starts, and there is nothing either keeps between calls.
#[derive(Default)]
pub(crate) struct Links {
    attached: bool,
}

impl Links {
    /// Lets URLs through, once, at boot.
    pub(crate) fn attach(&mut self) {
        self.attached = true;
    }

    /// Goes to `url`.
    ///
    /// On the web this replaces the page. Off it the desktop is asked to open the URL, which is
    /// what "go there" means where there is no page to replace.
    pub(crate) fn navigate(&self, url: &str) {
        debug!(url, "navigating");
        if !self.attached {
            return;
        }
        #[cfg(not(target_family = "wasm"))]
        if let Err(reason) = open::that_detached(url) {
            tracing::warn!(url, %reason, "navigation refused");
        }
        #[cfg(target_family = "wasm")]
        if let Some(window) = web_sys::window()
            && window.location().set_href(url).is_err()
        {
            tracing::warn!(url, "navigation refused");
        }
    }

    /// Asks the host to save what is at `url`.
    ///
    /// The web's, and only the web's: a browser is what turns a URL into a file in a person's
    /// downloads, and off it a program that wants a file on disk has one to write it with. Off the
    /// web this is traced and nothing else -- the surface is here on every target so that an app
    /// naming what it offers names it once.
    pub(crate) fn download(&self, url: &str) {
        debug!(url, "downloading");
        if !self.attached {
            return;
        }
        #[cfg(target_family = "wasm")]
        if let Some(anchor) = anchor(url) {
            // Clicked rather than navigated to: `download` is an attribute of the link and not of
            // the address, so there is no form of `location` that carries it.
            anchor.click();
            anchor.remove();
        }
    }
}

/// An anchor carrying `url`, in the page and ready to be clicked.
///
/// Built and taken out again around one click, because it is not something the page has -- it is
/// how the one verb the browser offers is spelled.
#[cfg(target_family = "wasm")]
fn anchor(url: &str) -> Option<web_sys::HtmlElement> {
    use wasm_bindgen::JsCast;

    let document = web_sys::window()?.document()?;
    let anchor = document
        .create_element("a")
        .ok()?
        .dyn_into::<web_sys::HtmlElement>()
        .ok()?;
    anchor.set_attribute("href", url).ok()?;
    anchor.set_attribute("download", "").ok()?;
    anchor.set_attribute("style", "display:none").ok()?;
    document.body()?.append_child(&anchor).ok()?;
    Some(anchor)
}
