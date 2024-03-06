use foliage::color::monochromatic::{AquaMarine, Monochromatic};
use foliage::color::Color;
use foliage::elm::Style;
use foliage::icon::FeatherIcon;
use foliage::r_scenes::circle_button::CircleButton;
use foliage::r_scenes::progress_bar::ProgressBar;
use foliage::segment::{MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

pub struct ProgressShowcase;
impl Viewable for ProgressShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);

    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.add_scene(
            ProgressBar::new(0.30, AquaMarine::BASE, Color::DARK_GREY),
            ResponsiveSegment::base(Segment::new(
                2.near().to(5.far()),
                2.near().to(2.far()).fixed(4.0),
            ))
            .at_layer(5),
        );
        view_builder.add_scene(
            CircleButton::new(
                FeatherIcon::Copy,
                Style::fill(),
                AquaMarine::BASE,
                Color::DARK_GREY,
            ),
            ResponsiveSegment::base(Segment::new(
                6.near().to(6.far()).maximum(50.0),
                2.near().to(2.far()),
            ))
            .at_layer(5),
        );
        view_builder.add_scene(
            ProgressBar::new(0.70, AquaMarine::BASE, Color::DARK_GREY),
            ResponsiveSegment::base(Segment::new(
                2.near().to(5.far()),
                3.near().to(3.far()).fixed(4.0),
            ))
            .at_layer(5),
        );
        view_builder.add_scene(
            CircleButton::new(
                FeatherIcon::Copy,
                Style::fill(),
                AquaMarine::BASE,
                Color::DARK_GREY,
            ),
            ResponsiveSegment::base(Segment::new(
                6.near().to(6.far()).maximum(50.0),
                3.near().to(3.far()),
            ))
            .at_layer(5),
        );
        view_builder.finish()
    }
}