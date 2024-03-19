use foliage::animate::Animate;
use foliage::circle_button::CircleButton;
use foliage::circle_progress_bar::CircleProgressBar;
use foliage::color::monochromatic::{AquaMarine, Greyscale, Monochromatic};
use foliage::coordinate::position::Position;
use foliage::coordinate::PositionAdjust;
use foliage::elm::Style;
use foliage::icon::FeatherIcon;
use foliage::progress_bar::{ProgressBar, ProgressPercent};
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::time::TimeDelta;
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};
use foliage::Colors;

pub struct ProgressShowcase;
impl Viewable for ProgressShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        let a = view_builder.add_scene(
            ProgressBar::new(0.30, Colors::new(AquaMarine::BASE, Greyscale::MINUS_THREE)),
            ResponsiveSegment::base(Segment::new(
                2.near().to(5.far()),
                2.near().to(2.far()).fixed(4.0),
            ))
            .at_layer(5),
        );
        let e = view_builder.add_scene(
            CircleButton::new(
                FeatherIcon::Copy,
                Style::fill(),
                Colors::new(AquaMarine::BASE, Greyscale::MINUS_THREE),
            ),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()).maximum(50.0),
                2.near().to(2.far()),
            ))
            .at_layer(5),
        );
        view_builder.add_command_to(
            e.root(),
            a.root().animate(
                Some(ProgressPercent(0.15)),
                ProgressPercent(1.0),
                TimeDelta::from_secs(1),
            ),
        );
        let b = view_builder.add_scene(
            CircleProgressBar::new(0.70, Colors::new(AquaMarine::BASE, Greyscale::MINUS_THREE)),
            ResponsiveSegment::base(Segment::new(
                2.near().to(5.far()).maximum(50.0),
                3.near().to(3.far()),
            ))
            .at_layer(5),
        );
        let d = view_builder.add_scene(
            CircleButton::new(
                FeatherIcon::Copy,
                Style::fill(),
                Colors::new(AquaMarine::BASE, Greyscale::MINUS_THREE),
            ),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()).maximum(50.0),
                3.near().to(3.far()),
            ))
            .at_layer(5),
        );
        view_builder.add_command_to(
            d.root(),
            b.root().animate(
                Some(ProgressPercent(0.15)),
                ProgressPercent(1.0),
                TimeDelta::from_secs(6),
            ),
        );
        view_builder.extend(b.root(), PositionAdjust(Position::new(-200.0, 0.0)));
        view_builder.add_command_to(
            d.root(),
            b.root().animate(
                Some(PositionAdjust(Position::new(-200.0, 0.0))),
                PositionAdjust(Position::default()),
                TimeDelta::from_secs(6),
            ),
        );
        view_builder.finish()
    }
}
