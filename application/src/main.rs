use foliage::action::{Actionable, ElmHandle};
use foliage::color::{Color, Grey, Monochromatic};
use foliage::grid::{Grid, GridCoordinate, GridPlacement};
use foliage::icon::{Icon, IconId};
use foliage::interaction::{ClickInteractionListener, OnClick};
use foliage::panel::{Panel, Rounding};
use foliage::Foliage;

#[derive(Clone)]
struct DeleteTest {}
impl Actionable for DeleteTest {
    fn apply(self, mut handle: ElmHandle) {
        handle.remove_element("first-sub-sub");
        handle.update_element("icon-change-test", |e| e.with_attr(IconId(1)));
        handle.update_attr_for("icon-change-test", |id: &mut IconId| id.0 = 2);
        // handle.remove_element("second");
    }
}
#[derive(Clone)]
struct OtherStuff {}
impl Actionable for OtherStuff {
    fn apply(self, mut handle: ElmHandle) {
        println!("other");
        handle.add_element(
            "first-sub",
            GridPlacement::new(1.col().to(1.col()), 1.row().to(1.row())).offset_layer(-1),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Grey::LIGHT))
                    .dependent_of("first")
            },
        );
        handle.add_element(
            "first-sub-sub",
            GridPlacement::new(1.col().to(1.col()), 1.row().to(1.row())).offset_layer(-1),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Grey::BASE))
                    .dependent_of("first-sub")
            },
        );
        handle.add_element(
            "first-sub-sub-sub",
            GridPlacement::new(1.col().to(1.col()), 1.row().to(1.row())).offset_layer(-1),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Grey::DARK))
                    .dependent_of("first-sub-sub")
            },
        );
        handle.create_signaled_action("click-test", DeleteTest {});
        handle.add_element(
            "first-sub-sub-sub-sub",
            GridPlacement::new(1.col().to(1.col()), 1.row().to(1.row())).offset_layer(-1),
            Option::from(Grid::new(1, 1)),
            |e| {
                e.with_attr(Panel::new(Rounding::default(), Color::BLACK))
                    .dependent_of("first-sub-sub-sub")
                    .with_attr(OnClick::new("click-test"))
                    .with_attr(ClickInteractionListener::new())
            },
        );
        handle.add_element(
            "icon-change-test",
            GridPlacement::new(1.col().to(1.col()), 1.row().to(1.row())).offset_layer(-1),
            None,
            |e| {
                e.with_attr(Icon::new(0, Color::BLACK))
                    .dependent_of("second")
            },
        )
    }
}
#[derive(Clone)]
struct Stuff {}
impl Actionable for Stuff {
    fn apply(self, mut handle: ElmHandle) {
        println!("stuff");
        handle.add_element(
            "first",
            GridPlacement::new(1.col().to(4.col()), 1.row().to(4.row())).offset_layer(4),
            Some(Grid::new(1, 1)),
            |e| e.with_attr(Panel::new(Rounding::default(), Color::WHITE)),
        );
        handle.add_element(
            "second",
            GridPlacement::new(5.col().to(8.col()), 1.row().to(4.row())).offset_layer(3),
            Some(Grid::new(4, 4)),
            |e| e.with_attr(Panel::new(Rounding::default(), Grey::BASE)),
        );
        handle.run_action(OtherStuff {});
        println!("almost-finished-stuff");
        // handle.remove_element("second");
    }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((800, 360));
    foliage.load_icon(0, include_bytes!("assets/icons/at-sign.icon"));
    foliage.load_icon(1, include_bytes!("assets/icons/grid.icon"));
    foliage.load_icon(2, include_bytes!("assets/icons/chevrons-left.icon"));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("");
    foliage.enable_signaled_action::<DeleteTest>();
    foliage.run_action(Stuff {});
    foliage.run();
}
