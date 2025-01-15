use bevy_ecs::component::Component;
use bevy_ecs::prelude::Resource;

use crate::AndroidConnection;

/// Adapter to interface with soft-input (VirtualKeyboard)
#[derive(Resource)]
pub struct VirtualKeyboardAdapter {
    #[allow(unused)]
    interface: AndroidConnection,
}

/// VirtualKeyboard Type for opening different pads on web/mobile
#[allow(unused)]
#[derive(Component, Copy, Clone)]
pub enum VirtualKeyboardType {
    Keyboard,
    TelephonePad,
    NumberPad,
}

impl VirtualKeyboardAdapter {
    #[allow(unused)]
    pub(crate) fn new(android_app: AndroidConnection) -> Self {
        Self::create_hook();
        Self {
            interface: android_app,
        }
    }
    #[allow(unused)]
    fn create_hook() {
        #[cfg(target_family = "wasm")]
        {
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
        }
    }
    #[allow(unused)]
    pub fn open(&self, ty: VirtualKeyboardType) {
        Self::trigger_hook(ty);
        #[cfg(target_os = "android")]
        {
            self.interface.0.as_ref().unwrap().show_soft_input(true);
            tracing::info!("opening keyboard");
        }
    }

    fn trigger_hook(_ty: VirtualKeyboardType) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::*;
            let document = web_sys::window().unwrap().document().unwrap();
            let trigger_element = match _ty {
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
            // trigger_element.blur().unwrap();
            trigger_element.focus().unwrap();
            web_sys::console::info_1(&JsValue::from_str("opening vkey"));
        }
    }
    #[allow(unused)]
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
            self.interface.0.as_ref().unwrap().hide_soft_input(true);
            tracing::info!("closing keyboard");
        }
    }
}
