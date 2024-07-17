use foliage::action::{Actionable, ElmHandle};
use foliage::color::{Color, Grey, Monochromatic};
use foliage::grid::{GridCoordinate, GridPlacement};
use foliage::panel::{Panel, Rounding};
use foliage::Foliage;

#[derive(Clone)]
struct OtherStuff {}
impl Actionable for OtherStuff {
    fn apply(self, mut handle: ElmHandle) {
        println!("other");

    }
}
#[derive(Clone)]
struct Stuff {}
impl Actionable for Stuff {
    fn apply(self, mut handle: ElmHandle) {
        println!("stuff");
        handle.add_element(
            "first",
            GridPlacement::new(1.span(4), 1.span(4)),
            None,
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
        handle.remove_element("second");
    }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((800, 360));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("");
    foliage.enable_signaled_action::<Stuff>();
    foliage.run_action(Stuff {});
    foliage.run();
}
