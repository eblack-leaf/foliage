use bevy_ecs::prelude::{Component, Resource};

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use crate::visualizer::{Attach, Visualizer};
#[cfg(target_os = "android")]
use crate::workflow::AndroidInterface;

/// Adapter to interface with soft-input (VirtualKeyboard)
#[derive(Resource)]
pub struct VirtualKeyboardAdapter {
    #[cfg(target_os = "android")]
    android_app: AndroidApp,
    #[cfg(not(target_os = "android"))]
    #[allow(unused)]
    android_app: (),
}

/// VirtualKeyboard Type for opening different pads on web
#[allow(unused)]
#[derive(Component, Copy, Clone)]
pub enum VirtualKeyboardType {
    Keyboard,
    TelephonePad,
    NumberPad,
}

impl VirtualKeyboardAdapter {
    #[cfg(target_family = "wasm")]
    pub(crate) fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let node = document.create_element("div").unwrap();
        node.set_inner_html(
            "<input type='text' maxlength='0' width=0 height=0 \
            id='keyboard_trigger' style='position: absolute;left: -1px;top: -1px;opacity: 0;\
            padding: 0;min-width: 0; min-height: 0;width: 0; height: 0;border: 0'>\
            <input type='tel' maxlength='0' width=0 height=0 \
            id='telephone_pad_trigger' style='position: absolute;left: -1px;top: -1px;opacity: 0;\
            padding: 0;min-width: 0; min-height: 0;width: 0; height: 0;border: 0'>\
            <input type='number' maxlength='0' width=0 height=0 \
            id='numpad_trigger' style='position: absolute;left: -1px;top: -1px;opacity: 0;\
            padding: 0;min-width: 0; min-height: 0;width: 0; height: 0;border: 0'>",
        );
        let body = document.body().unwrap();
        body.append_child(&node).unwrap();
        Self { android_app: () }
    }
    #[cfg(target_os = "android")]
    pub(crate) fn new(android_app: AndroidApp) -> Self {
        Self { android_app }
    }
    #[cfg(all(not(target_family = "wasm"), not(target_os = "android")))]
    pub(crate) fn new() -> Self {
        Self { android_app: () }
    }
    #[allow(unused)]
    pub fn open(&self, ty: VirtualKeyboardType) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{prelude::*, JsCast};
            let document = web_sys::window().unwrap().document().unwrap();
            let trigger_element = match ty {
                VirtualKeyboardType::Keyboard => document
                    .get_element_by_id("keyboard_trigger")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlElement>()
                    .unwrap(),
                VirtualKeyboardType::TelephonePad => document
                    .get_element_by_id("telephone_pad_trigger")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlElement>()
                    .unwrap(),
                VirtualKeyboardType::NumberPad => document
                    .get_element_by_id("numpad_trigger")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlElement>()
                    .unwrap(),
            };
            trigger_element.blur().unwrap();
            trigger_element.focus().unwrap();
            web_sys::console::info_1(&JsValue::from_str("opening vkey"));
        }
        #[cfg(target_os = "android")]
        {
            self.android_app.show_soft_input(true);
            info!("opening keyboard");
        }
    }
    pub fn close(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{JsCast, JsValue};
            let document = web_sys::window().unwrap().document().unwrap();
            document
                .get_element_by_id("keyboard_trigger")
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap()
                .blur()
                .unwrap();
            document
                .get_element_by_id("telephone_pad_trigger")
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap()
                .blur()
                .unwrap();
            document
                .get_element_by_id("numpad_trigger")
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap()
                .blur()
                .unwrap();
            web_sys::console::info_1(&JsValue::from_str("closing vkey"));
        }
        #[cfg(target_os = "android")]
        {
            self.android_app.hide_soft_input(true);
            info!("closing keyboard");
        }
    }
}

pub(crate) struct VirtualKeyboardAttachment;
impl Attach for VirtualKeyboardAttachment {
    fn attach(visualizer: &mut Visualizer) {
        #[cfg(not(target_os = "android"))]
        visualizer
            .job
            .container
            .insert_resource(VirtualKeyboardAdapter::new());
        #[cfg(target_os = "android")]
        {
            let app = visualizer
                .job
                .container
                .get_resource::<AndroidInterface>()
                .expect("android interface")
                .0
                .clone();
            visualizer
                .job
                .container
                .insert_resource(VirtualKeyboardAdapter::new(app));
        }
    }
}
