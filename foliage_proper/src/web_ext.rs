use crate::action::{Actionable, ElmHandle};

#[derive(Clone)]
pub struct HrefLink {
    href: String,
}
impl Actionable for HrefLink {
    fn apply(self, _handle: ElmHandle) {
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
