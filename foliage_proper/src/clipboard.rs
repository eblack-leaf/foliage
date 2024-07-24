use bevy_ecs::system::Resource;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Resource)]
pub struct ClipboardHandle {
    write_message: Option<String>,
}
impl ClipboardHandle {
    pub(crate) fn new() -> Self {
        Self {
            write_message: None,
        }
    }
    pub fn write<S: AsRef<str>>(&mut self, s: S) {
        self.write_message.replace(s.as_ref().to_string());
    }
    pub fn write_message(&mut self) -> Option<String> {
        self.write_message.take()
    }
}
pub fn clipboard_write(message: String) {
    #[cfg(not(target_family = "wasm"))]
    {
        // copypasta
    }
    #[cfg(target_family = "wasm")]
    {
        use js_sys::{wasm_bindgen, Array, Object, Reflect};
        use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{console, js_sys, Blob, BlobPropertyBag, Clipboard};

        thread_local! {
            static CLIPBOARD: Clipboard = web_sys::window().unwrap().navigator().clipboard().unwrap();
        }
        let data: Array = {
            let blob = Blob::new_with_blob_sequence_and_options(
                &Array::of1(&message.into()),
                BlobPropertyBag::new().type_("text/plain"),
            )
            .unwrap();
            let record = Object::new();
            Reflect::set(&record, &"text/plain".into(), &blob).unwrap();
            let item = ClipboardItemExt::new(&record);

            Array::of1(&item)
        };
        let promise = CLIPBOARD.with(|clipboard| clipboard.write(&data.into()));

        wasm_bindgen_futures::spawn_local(async move {
            if let Err(error) = JsFuture::from(promise).await {
                console::error_2(&"writing to clipboard failed: ".into(), &error);
            }
        });
    }
}

#[wasm_bindgen]
extern "C" {
    type ClipboardItemExt;

    #[wasm_bindgen(constructor, js_class = ClipboardItem)]
    fn new(data: &wasm_bindgen::JsValue) -> ClipboardItemExt;
}
