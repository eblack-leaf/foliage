use foliage::action::{Actionable, ElmHandle};
use foliage::anim::{Ease, SequenceTiming};
use foliage::bevy_ecs;
use foliage::bevy_ecs::system::Resource;
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
        handle.get_resource_mut::<Counter>().0 += 1;
        let i = handle.get_resource::<Counter>().0;
        handle.update_attr_for(
            TargetHandle::from("button-test").extend("text"),
            |t: &mut TextValue| {
                t.0 = format!("click-{}", i);
            },
        );
        let on_off = i % 3;
        handle.run_sequence(|seq| {
            let placement = if on_off == 0 {
                GridPlacement::new(2.col().to(4.col()), 2.row().to(2.row()))
            } else if on_off == 1 {
                GridPlacement::new(1.col().to(2.col()), 1.row().span(50))
            } else {
                GridPlacement::new(3.col().to(5.col()), 1.row().to(1.row()))
            };
            seq.animate_grid_placement(
                "button-test",
                placement.offset_layer(5),
                0.sec().to(500.millis()),
                Ease::DECELERATE,
            );
        });
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
            GridPlacement::new(1.col().to(2.col()), 1.row().span(50)).offset_layer(5),
            Button::new(
                0,
                "click",
                20,
                Coloring::new(Grey::minus_two(), Grey::BASE, Grey::BASE),
                OnClick::new("other-stuff"),
            )
            .rounded(Rounding::all(0.1)),
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
