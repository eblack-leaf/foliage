use bevy_ecs::component::Component;
use bevy_ecs::prelude::{IntoScheduleConfigs, Resource};
use bevy_ecs::system::Commands;

use crate::foliage::MainMarkers;
use crate::{AndroidConnection, Attachment, Foliage};

/// Text committed via the hidden trigger inputs' native `input` events -- winit's web backend
/// never fires `WindowEvent::Ime` (the canvas can't receive `CompositionEvent`), so this is the
/// only way typed/composed text from a mobile soft keyboard reaches the app at all. A `NonSend`
/// resource, not a regular one: wasm32 in the browser is single-threaded, so a plain
/// `Rc<RefCell<_>>` shared with the DOM closures is enough -- no `Arc`/`Mutex` needed.
#[cfg(target_family = "wasm")]
#[derive(Clone)]
pub(crate) struct VirtualInputQueue(
    std::rc::Rc<std::cell::RefCell<std::collections::VecDeque<String>>>,
);

/// Adapter to interface with soft-input (VirtualKeyboard)
#[derive(Resource)]
pub struct VirtualKeyboardAdapter {
    #[allow(unused)]
    interface: AndroidConnection,
}
impl Attachment for VirtualKeyboardAdapter {
    fn attach(foliage: &mut Foliage) {
        #[cfg(target_family = "wasm")]
        {
            let queue = VirtualInputQueue(std::rc::Rc::new(std::cell::RefCell::new(
                std::collections::VecDeque::new(),
            )));
            VirtualKeyboardAdapter::create_hook(queue.clone());
            foliage.world.insert_non_send(queue);
        }
        foliage
            .world
            .insert_resource(VirtualKeyboardAdapter::new(foliage.android_connection));
        foliage
            .main
            .add_systems(VirtualKeyboardAdapter::drain_virtual_input.in_set(MainMarkers::External));
    }
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
        Self {
            interface: android_app,
        }
    }
    /// Builds the three hidden trigger inputs and wires each one's native `input` event to
    /// forward committed text into `queue` -- previously only `.focus()`/`.blur()` existed
    /// (summons the OS soft keyboard), with nothing reading back what got typed through it.
    #[cfg(target_family = "wasm")]
    fn create_hook(queue: VirtualInputQueue) {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        let document = web_sys::window().unwrap().document().unwrap();
        let node = document.create_element("div").unwrap();
        node.set_inner_html(
            "<input type='text' maxlength='8192' width=0 height=0 \
            id='keyboard_trigger' style='position: absolute;left: -1px;top: -1px;opacity: 0;\
            padding: 0;min-width: 0; min-height: 0;width: 0; height: 0;border: 0'>\
            <input type='tel' maxlength='8192' width=0 height=0 \
            id='telephone_pad_trigger' style='position: absolute;left: -1px;top: -1px;opacity: 0;\
            padding: 0;min-width: 0; min-height: 0;width: 0; height: 0;border: 0'>\
            <input type='number' maxlength='8192' width=0 height=0 \
            id='numpad_trigger' style='position: absolute;left: -1px;top: -1px;opacity: 0;\
            padding: 0;min-width: 0; min-height: 0;width: 0; height: 0;border: 0'>",
        );
        let body = document.body().unwrap();
        body.append_child(&node).unwrap();

        for id in ["keyboard_trigger", "telephone_pad_trigger", "numpad_trigger"] {
            let queue = queue.clone();
            let element = document.get_element_by_id(id).unwrap();
            let on_input = Closure::wrap(Box::new(move |e: web_sys::Event| {
                if let Some(target) = e.target() {
                    if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                        let value = input.value();
                        if !value.is_empty() {
                            queue.0.borrow_mut().push_back(value);
                            input.set_value("");
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);
            element
                .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
                .unwrap();
            on_input.forget();
        }
    }
    /// Drains text captured from the hidden trigger inputs (see `create_hook`) into the same
    /// `InputSequence`/`Key::Character` path native `WindowEvent::Ime` commit already uses
    /// (`photosynthesis.rs`), so `TextInput`'s focus-routing picks it up for free.
    #[allow(unused_mut, unused_variables)]
    fn drain_virtual_input(
        mut commands: Commands,
        #[cfg(target_family = "wasm")] queue: bevy_ecs::system::NonSend<VirtualInputQueue>,
    ) {
        #[cfg(target_family = "wasm")]
        {
            let texts: Vec<String> = queue.0.borrow_mut().drain(..).collect();
            for text in texts {
                commands.trigger(crate::InputSequence::new(
                    crate::Key::Character(text),
                    crate::Modifiers::default(),
                ));
            }
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
