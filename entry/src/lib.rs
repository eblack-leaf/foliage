#[no_mangle]
#[cfg(target_os = "ios")]
pub extern "C" fn ios_main() {
    example::entry();
}

// android app hook

// web worker hook