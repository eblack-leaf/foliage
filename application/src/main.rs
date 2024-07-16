use foliage::action::{Actionable, ElmHandle};
use foliage::color::Color;
use foliage::grid::{GridCoordinate, GridPlacement};
use foliage::panel::{Panel, Rounding};
use foliage::Foliage;

#[derive(Clone)]
struct OtherStuff {}
impl Actionable for OtherStuff {
    fn apply(self, handle: ElmHandle) {
        println!("other");
    }
}
#[derive(Clone)]
struct Stuff {}
impl Actionable for Stuff {
    fn apply(self, mut handle: ElmHandle) {
        println!("stuff");
        handle.add_element("first", |e| {
            e.with_attr(GridPlacement::new(1.span(4), 1.span(4)))
                .with_attr(Panel::new(Rounding::default(), Color::WHITE))
        });
        handle.run_action(OtherStuff {});
        println!("almost-finished-stuff");
        handle.add_element("second", |e| e.with_attr(()))
    }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((360, 800));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("");
    foliage.enable_signaled_action::<Stuff>();
    foliage.run_action(Stuff {});
    foliage.run();
}
