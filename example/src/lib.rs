use foliage::panel::Panel;
use foliage::window::WindowDescriptor;
use foliage::Foliage;

pub fn entry() {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((400, 700)),
        )
        .with_leaf::<Panel>()
        .with_renderer::<Panel>()
        .run();
}
