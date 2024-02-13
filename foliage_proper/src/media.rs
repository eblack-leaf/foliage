use crate::differential::Despawn;
use crate::elm::config::CoreSet;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use crate::interaction::{InteractionListener, InteractionShape};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, IntoSystemConfigs};
use bevy_ecs::system::Query;
use compact_str::CompactString;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::Closure;
#[cfg(target_family = "wasm")]
use wasm_bindgen::JsCast;

#[derive(Component, Clone, Default)]
pub struct Href(CompactString, bool);

impl Href {
    pub(crate) fn absolute(&self) -> bool {
        self.1
    }
}

impl Href {
    pub fn new<S: AsRef<str>>(s: S, abs: bool) -> Self {
        Self(CompactString::from(s.as_ref()), abs)
    }
    pub fn link(&self) -> &str {
        self.0.as_str()
    }
}
#[derive(Bundle, Clone)]
pub struct HrefLink {
    href: Href,
    listener: InteractionListener,
}
impl HrefLink {
    pub fn absolute<S: AsRef<str>>(href: S) -> Self {
        Self {
            href: Href::new(href, true),
            listener: InteractionListener::default(),
        }
    }
    pub fn relative<S: AsRef<str>>(href: S) -> Self {
        Self {
            href: Href::new(href, false),
            listener: Default::default(),
        }
    }
    pub fn with_circle_shape(mut self) -> Self {
        self.listener =
            InteractionListener::default().with_shape(InteractionShape::InteractiveCircle);
        self
    }
}
fn navigation(hrefs: Query<(&Href, &InteractionListener, &Despawn)>) {
    for (href, listener, despawn) in hrefs.iter() {
        if despawn.should_despawn() {
            continue;
        }
        if listener.active() {
            Media::navigate_to(href.link(), href.absolute());
        }
    }
}
impl Leaf for Media {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems(navigation.in_set(CoreSet::ProcessEvent));
    }
}
pub struct Media {}
impl Media {
    #[allow(unused)]
    const ELEMENT_ID: &'static str = "media-overlay";
    #[allow(unused)]
    const BUTTON_HANDLE: &'static str = "media-overlay-trigger";
    #[allow(unused)]
    pub fn navigate_to(href: &str, absolute: bool) {
        #[cfg(target_family = "wasm")]
        {
            if let Some(window) = web_sys::window() {
                let origin = window.origin();
                let url = if absolute {
                    href.to_string()
                } else {
                    format!("{}{}", origin, href)
                };
                let document = window.document();
                if let Some(document) = document {
                    let node = document.create_element("div").unwrap();
                    let html = format!("<a href={} id='navigate-trigger'>", url);
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
