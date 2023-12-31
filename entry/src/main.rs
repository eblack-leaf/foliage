fn main() {
    let subscriber = tracing_subscriber::fmt();
    #[cfg(target_family = "wasm")]
    {
        console_error_panic_hook::set_once();
        subscriber
            .with_writer(
                tracing_subscriber_wasm::MakeConsoleWriter::default()
                    .map_trace_level_to(tracing::Level::DEBUG),
            )
            .without_time()
            .init();
    }
    #[cfg(not(target_family = "wasm"))]
    subscriber.init();
    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    example::entry(());
    #[cfg(target_os = "ios")]
    entry_lib::ios_main();
}
