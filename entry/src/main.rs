fn main() {
    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    example::entry(());
    #[cfg(target_os = "ios")]
    entry_lib::ios_main();
}
