use foliage::Foliage;

fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((800, 360));
    foliage.load_icon(0, include_bytes!("assets/icons/at-sign.icon"));
    foliage.load_icon(1, include_bytes!("assets/icons/grid.icon"));
    foliage.load_icon(2, include_bytes!("assets/icons/chevrons-left.icon"));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("foliage");
    foliage.run();
}
