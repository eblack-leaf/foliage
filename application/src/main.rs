use foliage::action::{Actionable, ElmHandle};
use foliage::anim::{EasementBehavior, SequenceTiming};
use foliage::bevy_ecs;
use foliage::bevy_ecs::system::Resource;
use foliage::clipboard::ClipboardHandle;
use foliage::color::{Grey, Monochromatic};
use foliage::element::TargetHandle;
use foliage::grid::{GridCoordinate, GridPlacement};
use foliage::interaction::OnClick;
use foliage::panel::Rounding;
use foliage::style::Coloring;
use foliage::text::TextValue;
use foliage::view::button::Button;
use foliage::Foliage;

#[derive(Clone)]
struct ButtonTest {}
#[derive(Resource)]
struct Counter(i32);
impl Actionable for ButtonTest {
    fn apply(self, mut handle: ElmHandle) {
        println!("hello-world");
        handle.get_resource_mut::<Counter>().0 += 1;
        let i = handle.get_resource::<Counter>().0;
        handle.update_attr_for(
            TargetHandle::from("button-test").extend("text"),
            |t: &mut TextValue| {
                println!("text-val: {}", t.0);
                t.0 = format!("click-{}", i);
            },
        );
        let i = if i >= 4 {
            let diff = i - 4;
            diff
        } else {
            i
        };
        handle.run_sequence(|seq| {
            seq.animate_grid_placement(
                "button-test",
                GridPlacement::new(
                    (1 + i).col().to((2 + i).col()),
                    (1 + i / 2).row().to((1 + i / 2).row()),
                )
                .offset_layer(5),
                1.sec().to(1500.millis()),
                EasementBehavior::Linear,
            );
        });
        handle
            .get_resource_mut::<ClipboardHandle>()
            .write("howdy-there");
    }
}
#[derive(Clone)]
struct Stuff {}
impl Actionable for Stuff {
    fn apply(self, mut handle: ElmHandle) {
        handle.add_resource(Counter(0));
        handle.create_signaled_action("other-stuff", ButtonTest {});
        handle.add_view(
            None,
            "button-test",
            GridPlacement::new(1.col().to(2.col()), 1.row().to(1.row())).offset_layer(5),
            Button::new(0, "click", 20, OnClick::new("other-stuff"))
                .rounded(Rounding::all(0.1))
                .colored(Coloring::new(Grey::LIGHT, Grey::DARK, Grey::LIGHT)),
        );
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
    foliage.set_base_url("foliage");
    foliage.enable_signaled_action::<ButtonTest>();
    foliage.run_action(Stuff {});
    foliage.run();
}
