//! Where bytes come from, and what happens when they arrive.
//!
//! Everything foliage is given is bytes: a font, an icon's field, a picture. What is here is the
//! road from a file or a URL to them, and it is deliberately the *only* difference between the two
//! cases -- [`font`](crate::Grove::font), [`icon`](crate::Grove::icon) and
//! [`image`](crate::Grove::image) each take what they are given or where to go and get it, and hand
//! back the same name either way. An app that fetches writes the same line as one that bundles.
//!
//! An arrival is an **op**, pushed onto the one queue from wherever it finished. That is what keeps
//! it on F1's footing: ordered by when it arrived like every other change, drained where every other
//! change is drained, and never written into a frame from the side.

use crate::image::Plate;
use crate::op::Op;
use crate::queue::{Queue, Wake};
use crate::text::Font;

/// Where bytes an app does not already hold are read from.
///
/// A path exists only where there is a filesystem, so naming one on the web is a compile error
/// rather than a warning at runtime. A URL is nameable everywhere, and answered on the web.
///
/// A URL is used exactly as given. Composing an origin or a base is the app's, because where its
/// assets are hosted is a fact about its deployment and not about the engine.
#[derive(Clone, Debug)]
pub enum Origin {
    /// A file, at a path this program can read.
    #[cfg(not(target_family = "wasm"))]
    Path(std::path::PathBuf),
    /// A URL, as the platform resolves it.
    Url(String),
}

impl Origin {
    /// A file to read.
    #[cfg(not(target_family = "wasm"))]
    pub fn path(path: impl Into<std::path::PathBuf>) -> Self {
        Self::Path(path.into())
    }

    /// A URL to fetch.
    ///
    /// Fetched on the web. **Off it, one is accepted and reported as
    /// [`missing`](crate::Pollen::missing)**: an http client and a TLS stack are a large addition to
    /// a dependency list, and nothing has yet asked for one. The name is here on every target so
    /// that a program stating its assets' URLs states them once, and so what is outstanding is an
    /// implementation rather than a surface.
    pub fn url(url: impl Into<String>) -> Self {
        Self::Url(url.into())
    }
}

/// What a registration is given: bytes an app holds, or an [`Origin`] to read them from.
///
/// Never named at a callsite -- every verb that takes one takes `impl Into<Bytes>`, so
/// `include_bytes!(..)` and `Origin::path(..)` are both simply passed.
#[derive(Clone, Debug)]
pub struct Bytes(pub(crate) Supply);

#[derive(Clone, Debug)]
pub(crate) enum Supply {
    /// Here now. Registered in the call that was handed them.
    Held(Vec<u8>),
    /// Somewhere else. Registered in the frame they arrive in.
    At(Origin),
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(Supply::Held(bytes))
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self(Supply::Held(bytes.to_vec()))
    }
}

impl From<&Vec<u8>> for Bytes {
    fn from(bytes: &Vec<u8>) -> Self {
        Self(Supply::Held(bytes.clone()))
    }
}

/// What `include_bytes!` produces, which is the bundled case and so the one that has to be free.
impl<const N: usize> From<&[u8; N]> for Bytes {
    fn from(bytes: &[u8; N]) -> Self {
        Self(Supply::Held(bytes.to_vec()))
    }
}

impl From<Origin> for Bytes {
    fn from(origin: Origin) -> Self {
        Self(Supply::At(origin))
    }
}

/// What a retrieval fills once its bytes land.
///
/// Carried on the arrival op, because the name was handed out before anything was read and the
/// drain has to know what it is filling.
#[derive(Copy, Clone, Debug)]
pub(crate) enum Destination {
    Face(Font),
    /// A mark, with the two numbers only the app knows about the field it is fetching.
    Mark(crate::icon::Field, u32, f32),
    Picture(Plate),
}

/// Something named before its bytes existed, which can now be asked whether they arrived.
///
/// One question over three handles: a [`Font`], a [`Field`](crate::Field) and a [`Plate`] are filled
/// by the same road, so [`loaded`](crate::Pollen::loaded) and [`missing`](crate::Pollen::missing)
/// ask about them the same way and an app that waits on several waits on them alike.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Arrival {
    /// A typeface.
    Face(Font),
    /// A mark.
    Mark(crate::icon::Field),
    /// A picture.
    Picture(Plate),
}

impl From<Font> for Arrival {
    fn from(font: Font) -> Self {
        Self::Face(font)
    }
}

impl From<crate::icon::Field> for Arrival {
    fn from(field: crate::icon::Field) -> Self {
        Self::Mark(field)
    }
}

impl From<Plate> for Arrival {
    fn from(plate: Plate) -> Self {
        Self::Picture(plate)
    }
}

impl From<Destination> for Arrival {
    fn from(destination: Destination) -> Self {
        match destination {
            Destination::Face(font) => Self::Face(font),
            Destination::Mark(field, _, _) => Self::Mark(field),
            Destination::Picture(plate) => Self::Picture(plate),
        }
    }
}

/// What a retrieval finished with: the bytes, or why there are none.
pub(crate) type Retrieved = Result<Vec<u8>, String>;

/// Starts a retrieval, whichever road the target has.
///
/// The name has already been handed to the app by the time this runs, which is the whole point: what
/// is waited on is nameable before it arrives, so everything that draws it is written now.
pub(crate) fn retrieve(queue: &Queue, wake: &Wake, destination: Destination, origin: Origin) {
    let (queue, wake) = (queue.clone(), wake.clone());
    match origin {
        #[cfg(not(target_family = "wasm"))]
        Origin::Path(path) => {
            // Off the calling thread even for a local file. A read is not instant because it is
            // local -- a font is megabytes -- and the thread that called this is the one drawing.
            std::thread::spawn(move || {
                let bytes = std::fs::read(&path).map_err(|failed| failed.to_string());
                arrived(&queue, &wake, destination, bytes);
            });
        }
        // TODO: native http, behind a feature that brings the client and the TLS stack with it.
        // The arrival is what would change; everything either side of it is already in place.
        #[cfg(not(target_family = "wasm"))]
        Origin::Url(url) => {
            arrived(
                &queue,
                &wake,
                destination,
                Err(format!("a URL is only fetched on the web: {url}")),
            );
        }
        #[cfg(target_family = "wasm")]
        Origin::Url(url) => {
            wasm_bindgen_futures::spawn_local(async move {
                let bytes = fetch(&url).await;
                arrived(&queue, &wake, destination, bytes);
            });
        }
    }
}

/// Queues an arrival and asks for the frame that will drain it.
///
/// The whole of what a retrieval does when it finishes, so every one of them lands the same way
/// whatever read the bytes.
fn arrived(queue: &Queue, wake: &Wake, destination: Destination, bytes: Retrieved) {
    queue.push(Op::Arrived { destination, bytes });
    wake.rouse();
}

/// The fetch itself, as a sequence of promises.
///
/// Every failure along it is one failure to an app -- the bytes did not arrive -- so each is named
/// for the trace and none is distinguished in what is reported.
#[cfg(target_family = "wasm")]
async fn fetch(url: &str) -> Retrieved {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or("no window")?;
    let response = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|_| "the fetch failed".to_string())?
        .dyn_into::<web_sys::Response>()
        .map_err(|_| "not a response".to_string())?;
    if !response.ok() {
        return Err(format!("answered {}", response.status()));
    }
    let body = JsFuture::from(response.array_buffer().map_err(|_| "no body".to_string())?)
        .await
        .map_err(|_| "the body failed".to_string())?;
    Ok(js_sys::Uint8Array::new(&body).to_vec())
}
