use tracing::metadata::LevelFilter;
use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

fn main() {
    let targets = Targets::new()
        .with_target("foliage", Level::TRACE)
        .with_target("foliage::texture", Level::TRACE)
        .with_target("foliage::differential", Level::TRACE)
        // .with_target("foliage::ash", LevelFilter::OFF)
        .with_target("example", Level::TRACE)
        .with_target("entry", Level::TRACE);
    #[cfg(not(target_family = "wasm"))]
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(targets))
        .init();
    #[cfg(target_family = "wasm")]
    {
        console_error_panic_hook::set_once();
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(
                        tracing_subscriber_wasm::MakeConsoleWriter::default()
                            .map_trace_level_to(tracing::Level::TRACE),
                    )
                    .without_time()
                    .with_filter(targets),
            )
            .init();
    }

    #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
    example::entry(());
    #[cfg(target_os = "ios")]
    entry_lib::ios_main();
}