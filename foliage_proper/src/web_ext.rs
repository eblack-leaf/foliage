#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;

/// A URL the app can navigate to. No-op off the web -- the whole type compiles away to
/// nothing on native, so callers need no `cfg` of their own.
#[derive(Clone)]
#[allow(unused)]
pub struct HrefLink {
    href: String,
}
#[allow(unused)]
impl HrefLink {
    /// A link to `s`.
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self {
            href: s.as_ref().to_string(),
        }
    }
    /// Navigates the page to this URL, by synthesizing and clicking a hidden anchor --
    /// an anchor click, unlike assigning `location`, is what browsers treat as
    /// user-initiated, so it is not blocked as a popup.
    pub fn navigate(&self) {
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    use wasm_bindgen::JsCast;
                    let node = document.create_element("div").unwrap();
                    let html = format!("<a href={} id='navigate-trigger'>", self.href);
                    node.set_id("navigate-trigger-div");
                    node.set_inner_html(html.as_str());
                    document.body().unwrap().append_child(&node).unwrap();
                    let html_element = document
                        .get_element_by_id("navigate-trigger")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlElement>()
                        .unwrap();
                    html_element.click();
                    html_element.remove();
                    document
                        .get_element_by_id("navigate-trigger-div")
                        .unwrap()
                        .remove();
                }
            }
        }
    }
}

/// Browser capabilities with no engine equivalent -- downloads, and playing media in a
/// DOM overlay above the canvas.
///
/// Every method is a no-op off the web, so calling code stays `cfg`-free. Media plays in
/// a real DOM element rather than being rendered by the engine: the browser owns the
/// codecs and the controls.
pub struct Extensions {}
impl Extensions {
    #[allow(unused)]
    const ELEMENT_ID: &'static str = "media-overlay";
    #[allow(unused)]
    const BUTTON_HANDLE: &'static str = "media-overlay-trigger";
    #[allow(unused)]
    /// Prompts the browser to download `href`.
    pub fn download(href: &str) {
        #[cfg(target_family = "wasm")]
        {
            let document = web_sys::window().unwrap().document().unwrap();
            let node = document.create_element("div").unwrap();
            let html = format!("<a href={} id='download-trigger' download>", href);
            node.set_id("download-trigger-div");
            node.set_inner_html(html.as_str());
            document.body().unwrap().append_child(&node).unwrap();
            let html_element = document
                .get_element_by_id("download-trigger")
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap();
            html_element.click();
            html_element.remove();
            document
                .get_element_by_id("download-trigger-div")
                .unwrap()
                .remove();
        }
    }
    #[allow(unused)]
    /// Opens `src` as a `<video>` in the overlay, `ty` being its MIME type. Dismiss with
    /// [`remove`](Self::remove).
    pub fn web_video(src: &str, ty: &str) {
        #[cfg(target_family = "wasm")]
        {
            let element_html = format!(
                "<video style=\"height:95%;width:auto\" controls>
                <source src={} type={}>
            </video>",
                src, ty
            );
            Self::media_overlay(element_html);
            return;
        }
    }
    #[cfg(not(target_family = "wasm"))]
    #[allow(unused)]
    /// Native counterpart to [`web_video`](Self::web_video): hands `src` to the system's
    /// own player.
    pub fn native_video(src: &str) {
        #[cfg(not(target_family = "wasm"))]
        {
            let _ = open::that(src);
        }
    }
    #[allow(unused)]
    /// Opens `src` as an embedded document in the overlay. Dismiss with
    /// [`remove`](Self::remove).
    pub fn web_document(src: &str) {
        #[cfg(target_family = "wasm")]
        {
            let element_html = format!(
                "
        <iframe src={} style=\"height:95%;width:95%\">
        </iframe>
        ",
                src
            );
            Self::media_overlay(element_html);
            return;
        }
    }
    #[allow(unused)]
    /// Native counterpart to [`web_document`](Self::web_document): opens `src` in the
    /// system's own viewer.
    pub fn native_document(src: &str) {
        #[cfg(not(target_family = "wasm"))]
        {
            open::that(src);
        }
    }
    #[allow(unused)]
    fn media_overlay(element_html: String) {
        #[cfg(target_family = "wasm")]
        {
            let document = web_sys::window().unwrap().document().unwrap();
            let node = document.create_element("div").unwrap();
            node.set_id(Self::ELEMENT_ID);
            let html = format!(
                "
        <div style=\"
            display:flex;
            justify-content:center; width: 100%;height: 100%; padding:5px;
            background: black; position: absolute; top: 0;left: 0\">
            {}
        </div>
        <button id={} style=\"
                    position:absolute;
                    top:0;
                    left:0;
                    width:40px;
                    height:40px;
                    border:none;
                    color:white;
                    background:black;
                    text-align:center;
                    text-decoration:none;
                    font-size:32px;\">&times
        </button>",
                element_html,
                Self::BUTTON_HANDLE
            );
            node.set_inner_html(html.as_str());
            let body = document.body().unwrap();
            body.append_child(&node).unwrap();
            let callback = Closure::wrap(Box::new(move |_e: web_sys::Event| {
                Self::remove();
            }) as Box<dyn FnMut(_)>);
            document
                .get_element_by_id(Self::BUTTON_HANDLE)
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap()
                .set_onclick(Some(callback.as_ref().unchecked_ref()));
            callback.forget();
        }
    }
    #[allow(unused)]
    /// Tears down the media overlay, restoring the canvas underneath.
    pub fn remove() {
        #[cfg(target_family = "wasm")]
        {
            let document = web_sys::window().unwrap().document().unwrap();
            document
                .get_element_by_id(Self::ELEMENT_ID)
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap()
                .remove();
        }
    }
}
