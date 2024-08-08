use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::branch::{Branch, Tree};

#[derive(Clone)]
pub struct HrefLink {
    href: String,
}
impl Branch for HrefLink {
    fn grow(self, _handle: Tree) {
        self.navigate();
    }
}
impl HrefLink {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self {
            href: s.as_ref().to_string(),
        }
    }
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

pub struct Extensions {}
impl Extensions {
    #[allow(unused)]
    const ELEMENT_ID: &'static str = "media-overlay";
    #[allow(unused)]
    const BUTTON_HANDLE: &'static str = "media-overlay-trigger";
    #[allow(unused)]
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
    pub fn native_video(src: &str) {
        #[cfg(not(target_family = "wasm"))]
        {
            let _ = open::that(src);
        }
    }
    #[allow(unused)]
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
