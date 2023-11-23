#[no_mangle]
#[cfg(target_os = "ios")]
pub extern "C" fn ios_main() {
    example::entry(());
}

// android app hook
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: foliage::AndroidApp) {
    example::entry(foliage::AndroidInterface::new(app));
}
// web worker hook
