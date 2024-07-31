use foliage::action::{Actionable, ElmHandle};
use foliage::anim::{Ease, SequenceTiming};
use foliage::color::{Grey, Monochromatic};
use foliage::grid::{GridCoordinate, GridPlacement};
use foliage::interaction::OnClick;
use foliage::panel::{Panel, Rounding};
use foliage::style::Coloring;
use foliage::view::button::Button;
use foliage::{icon_handle, Foliage};

#[icon_handle]
enum IconHandles {
    Trigger,
}
#[derive(Clone)]
struct Shaping {}
#[derive(Clone)]
struct ShapeSequence {}
impl Actionable for ShapeSequence {
    fn apply(self, mut handle: ElmHandle) {
        handle.update_visibility("trigger", false);
        handle.run_sequence(|seq| {
            seq.animate_attr(
                "shape",
                GridPlacement::new(2.col().to(2.col()), 1.row().to(1.row())),
                0.millis().to(500.millis()),
                Ease::DECELERATE,
            );
            seq.animate_attr(
                "shape",
                GridPlacement::new(2.col().to(2.col()), 2.row().to(2.row())),
                750.millis().to(1250.millis()),
                Ease::ACCELERATE,
            );
            seq.animate_attr(
                "shape",
                GridPlacement::new(2.col().to(4.col()), 2.row().to(2.row())),
                1500.millis().to(2000.millis()),
                Ease::ACCELERATE,
            );
            seq.animate_attr(
                "shape",
                GridPlacement::new(2.col().to(4.col()), 2.row().to(4.row())),
                2250.millis().to(2750.millis()),
                Ease::ACCELERATE,
            );
            seq.animate_attr(
                "shape",
                GridPlacement::new(1.col().to(1.col()), 1.row().to(1.row())),
                3000.millis().to(3500.millis()),
                Ease::ACCELERATE,
            );
        });
    }
}
impl Actionable for Shaping {
    fn apply(self, mut handle: ElmHandle) {
        handle.create_signaled_action("shape-sequence", ShapeSequence {});
        handle.add_view(
            "trigger",
            GridPlacement::new(40.percent().to(60.percent()), 80.percent().to(90.percent())),
            2,
            Button::new(
                IconHandles::Trigger,
                Coloring::new(Grey::minus_two(), Grey::plus_two()),
                OnClick::new("shape-sequence"),
            ),
            None,
        );
        handle.add_element(
            "shape",
            GridPlacement::new(1.col().to(1.col()), 1.row().to(1.row())),
            1,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::plus_two()));
            },
        );

        handle.add_element(
            "back-1",
            GridPlacement::new(2.col().to(2.col()), 1.row().to(1.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-2",
            GridPlacement::new(3.col().to(3.col()), 1.row().to(1.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-3",
            GridPlacement::new(1.col().to(1.col()), 2.row().to(2.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(1.col().to(1.col()), 3.row().to(3.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-5",
            GridPlacement::new(2.col().to(2.col()), 3.row().to(3.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(3.col().to(3.col()), 3.row().to(3.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-5",
            GridPlacement::new(2.col().to(2.col()), 2.row().to(2.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-6",
            GridPlacement::new(3.col().to(3.col()), 2.row().to(2.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(4.col().to(4.col()), 1.row().to(1.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(4.col().to(4.col()), 2.row().to(2.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(4.col().to(4.col()), 3.row().to(3.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(4.col().to(4.col()), 4.row().to(4.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(1.col().to(1.col()), 4.row().to(4.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(2.col().to(2.col()), 4.row().to(4.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
        handle.add_element(
            "back-4",
            GridPlacement::new(3.col().to(3.col()), 4.row().to(4.row())),
            5,
            None,
            |e| {
                e.give_attr(Panel::new(Rounding::all(0.0), Grey::minus_two()));
            },
        );
    }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((800, 600));
    foliage.load_icon(
        IconHandles::Trigger,
        include_bytes!("assets/icons/grid.icon"),
    );
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("foliage");
    foliage.enable_signaled_action::<ShapeSequence>();
    foliage.run_action(Shaping {});
    foliage.run();
}
