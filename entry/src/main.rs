fn main() {
    #[cfg(not(target_os = "ios"))]
    example::entry();
    #[cfg(target_os = "ios")]
    entry_lib::ios_main();
}

