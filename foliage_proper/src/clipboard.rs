use wasm_bindgen::prelude::wasm_bindgen;

pub(crate) fn clipboard_test(message: String) {
    #[cfg(target_family = "wasm")] {
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