fn main() {
    #[cfg(target_family = "wasm")]
    console_log::init_with_level(Level::Info).expect("console-log");
    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    example::entry(());
    #[cfg(target_os = "ios")]
    entry_lib::ios_main();
}
