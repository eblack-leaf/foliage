use crate::ThemeColor;
use foliage::color::monochromatic::Monochromatic;
use foliage::color::Color;
use foliage::coordinate::CoordinateUnit;
use foliage::elm::Style;
use foliage::icon::FeatherIcon;
use foliage::layout::Layout;
use foliage::procedure::Procedure;
use foliage::r_scenes::button::Button;
use foliage::r_scenes::circle_button::CircleButton;
use foliage::r_scenes::icon_button::IconButton;
use foliage::r_scenes::icon_text::IconText;
use foliage::r_scenes::text_button::TextButton;
use foliage::scene::Scene;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, Text, TextValue};
use foliage::view::{ViewBuilder, ViewDescriptor, Viewable};

struct ButtonDisplay<T> {
    first: T,
    second: T,
    desc: String,
    row: i32,
    max_w: Option<CoordinateUnit>,
    max_h: Option<CoordinateUnit>,
}
impl<T> ButtonDisplay<T> {
    pub fn new(
        first: T,
        second: T,
        desc: String,
        row: i32,
        max_w: Option<CoordinateUnit>,
        max_h: Option<CoordinateUnit>,
    ) -> Self {
        Self {
            first,
            second,
            desc,
            row,
            max_w,
            max_h,
        }
    }
}
impl<T: Scene> Procedure for ButtonDisplay<T> {
    fn steps(self, view_builder: &mut ViewBuilder) {
        view_builder.add_scene(
            self.first,
            ResponsiveSegment::base(Segment::new(
                2.near()
                    .to(3.far())
                    .minimum(150.0)
                    .maximum(if let Some(m) = self.max_w { m } else { 5000.0 }),
                self.row
                    .near()
                    .to(self.row.far())
                    .maximum(if let Some(m) = self.max_h { m } else { 5000.0 }),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    1.near()
                        .to(4.far())
                        .minimum(100.0)
                        .maximum(if let Some(m) = self.max_w { m } else { 5000.0 }),
                    self.row
                        .near()
                        .to(self.row.far())
                        .maximum(if let Some(m) = self.max_h { m } else { 55.0 }),
                ),
            )
            .justify(Justify::Top)
            .at_layer(5),
        );
        view_builder.add_scene(
            self.second,
            ResponsiveSegment::base(Segment::new(
                5.near()
                    .to(6.far())
                    .minimum(150.0)
                    .maximum(if let Some(m) = self.max_w { m } else { 5000.0 }),
                self.row
                    .near()
                    .to(self.row.far())
                    .maximum(if let Some(m) = self.max_h { m } else { 5000.0 }),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    5.near()
                        .to(8.far())
                        .minimum(100.0)
                        .maximum(if let Some(m) = self.max_w { m } else { 5000.0 }),
                    self.row
                        .near()
                        .to(self.row.far())
                        .maximum(if let Some(m) = self.max_h { m } else { 55.0 }),
                ),
            )
            .justify(Justify::Top)
            .at_layer(5),
        );
        view_builder.add(
            Text::new(
                MaxCharacters(11),
                TextValue::new(self.desc),
                ThemeColor::MINUS_THREE,
            ),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()),
                self.row
                    .near()
                    .to(self.row.far())
                    .minimum(30.0)
                    .maximum(40.0),
            ))
            .without_portrait_mobile()
            .without_portrait_tablet()
            .justify(Justify::Top)
            .at_layer(5),
        );
    }
}
pub struct ButtonShowcase;
impl Viewable for ButtonShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 5);
    fn view(mut view_builder: ViewBuilder) -> ViewDescriptor {
        view_builder.apply(ButtonDisplay::new(
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                Style::fill(),
                Color::DARK_GREY,
                ThemeColor::MINUS_ONE,
            ),
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                Style::ring(),
                Color::DARK_GREY,
                ThemeColor::MINUS_ONE,
            ),
            "base".to_string(),
            2,
            None,
            Some(45.0),
        ));
        view_builder.apply(ButtonDisplay::new(
            TextButton::new(
                TextValue::new("copy"),
                MaxCharacters(4),
                Style::fill(),
                Color::BLACK,
                ThemeColor::MINUS_ONE,
            ),
            TextButton::new(
                TextValue::new("copy"),
                MaxCharacters(4),
                Style::ring(),
                Color::BLACK,
                ThemeColor::MINUS_ONE,
            ),
            "text".to_string(),
            3,
            None,
            Some(45.0),
        ));
        view_builder.apply(ButtonDisplay::new(
            CircleButton::new(
                FeatherIcon::Copy,
                Style::fill(),
                Color::DARK_GREY,
                ThemeColor::PLUS_ONE,
            ),
            CircleButton::new(
                FeatherIcon::Copy,
                Style::ring(),
                Color::BLACK,
                ThemeColor::PLUS_ONE,
            ),
            "circle".to_string(),
            4,
            Some(55.0),
            Some(55.0),
        ));
        view_builder.apply(ButtonDisplay::new(
            IconButton::new(
                FeatherIcon::Copy,
                Style::fill(),
                Color::DARK_GREY,
                ThemeColor::PLUS_THREE,
            ),
            IconButton::new(
                FeatherIcon::Copy,
                Style::ring(),
                Color::BLACK,
                ThemeColor::PLUS_THREE,
            ),
            "icon".to_string(),
            5,
            Some(45.0),
            Some(45.0),
        ));
        view_builder.finish()
    }
}
