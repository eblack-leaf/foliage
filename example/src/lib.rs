use foliage::color::Color;
use foliage::elm::{Elm, Leaf};
use foliage::panel::{Panel, PanelStyle};
use foliage::window::WindowDescriptor;
use foliage::Foliage;

pub fn entry() {
    Foliage::new()
        .with_window_descriptor(
            WindowDescriptor::new()
                .with_title("foliage")
                .with_desktop_dimensions((400, 700)),
        )
        .with_renderleaf::<Panel>()
        .with_leaf::<Tester>()
        .run();
}
struct Tester;
impl Leaf for Tester {
    fn attach(elm: &mut Elm) {
        elm.job.container.spawn(Panel::new(
            PanelStyle::flat(),
            (100, 100).into(),
            (200, 100).into(),
            2.into(),
            Color::OFF_WHITE.into(),
        ));
        elm.job.container.spawn(Panel::new(
            PanelStyle::ring(),
            (100, 300).into(),
            (200, 100).into(),
            2.into(),
            Color::OFF_WHITE.into(),
        ));
    }
}
