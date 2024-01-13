use crate::elm::config::ElmConfiguration;
use crate::elm::leaf::{EmptySetDescriptor, Leaf};
use crate::elm::Elm;
use crate::AndroidInterface;
use bevy_ecs::prelude::{Component, Resource};

/// Adapter to interface with soft-input (VirtualKeyboard)
#[derive(Resource)]
pub struct VirtualKeyboardAdapter {
    interface: AndroidInterface,
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
    pub(crate) fn new(android_app: AndroidInterface) -> Self {
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
        Self {
            interface: android_app,
        }
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
            self.interface.0.as_ref().unwrap().show_soft_input(true);
            tracing::info!("opening keyboard");
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
            self.interface.0.as_ref().unwrap().hide_soft_input(true);
            tracing::info!("closing keyboard");
        }
    }
}

impl Leaf for VirtualKeyboardAdapter {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        let interface = elm
            .container()
            .get_resource::<AndroidInterface>()
            .cloned()
            .unwrap();
        elm.container()
            .insert_resource(VirtualKeyboardAdapter::new(interface));
    }
}
