use foliage::action::{Actionable, ElmHandle};
use foliage::color::{Color, Grey, Monochromatic};
use foliage::grid::{Grid, GridCoordinate, GridPlacement};
use foliage::interaction::{ClickInteractionListener, OnClick};
use foliage::panel::{Panel, Rounding};
use foliage::Foliage;
#[derive(Clone)]
struct DeleteTest {}
impl Actionable for DeleteTest {
    fn apply(self, mut handle: ElmHandle) {
        handle.remove_element("first-sub-sub");
    }
}
#[derive(Clone)]
struct OtherStuff {}
impl Actionable for OtherStuff {
    fn apply(self, mut handle: ElmHandle) {
        println!("other");
        handle.add_element(
            "first-sub",
            GridPlacement::new(1.span(1), 1.span(1)).offset_layer(-1),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Grey::LIGHT))
                    .dependent_of("first")
            },
        );
        handle.add_element(
            "first-sub-sub",
            GridPlacement::new(1.span(1), 1.span(1)).offset_layer(-2),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Grey::BASE))
                    .dependent_of("first-sub")
            },
        );
        handle.add_element(
            "first-sub-sub-sub",
            GridPlacement::new(1.span(1), 1.span(1)).offset_layer(-3),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Grey::DARK))
                    .dependent_of("first-sub-sub")
            },
        );
        handle.create_signaled_action("click-test", DeleteTest {});
        handle.add_element(
            "first-sub-sub-sub-sub",
            GridPlacement::new(1.span(1), 1.span(1)).offset_layer(-4),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Color::BLACK))
                    .dependent_of("first-sub-sub-sub")
                    .with_attr(OnClick::new("click-test"))
                    .with_attr(ClickInteractionListener::new())
            },
        );
    }
}
#[derive(Clone)]
struct Stuff {}
impl Actionable for Stuff {
    fn apply(self, mut handle: ElmHandle) {
        println!("stuff");
        handle.add_element(
            "first",
            GridPlacement::new(1.span(4), 1.span(4)).offset_layer(10),
            Some(Grid::new(1, 1)),
            |e| e.with_attr(Panel::new(Rounding::default(), Color::WHITE)),
        );
        handle.run_action(OtherStuff {});
        println!("almost-finished-stuff");
        handle.add_element(
            "second",
            GridPlacement::new(5.span(4), 1.span(4)),
            None,
            |e| e.with_attr(Panel::new(Rounding::default(), Grey::BASE)),
        );
        // handle.remove_element("second");
    }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((800, 360));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("");
    foliage.enable_signaled_action::<DeleteTest>();
    foliage.run_action(Stuff {});
    foliage.run();
}
