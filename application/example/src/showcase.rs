use foliage::aesthetic::Aesthetic;
use foliage::bevy_ecs::prelude::Commands;
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::color::monochromatic::{Monochromatic, Orange as THEME_COLOR};
use foliage::color::Color;
use foliage::coordinate::CoordinateUnit;
use foliage::elm::ElementStyle;
use foliage::icon::FeatherIcon;
use foliage::layout::Layout;
use foliage::r_scenes::button::Button;
use foliage::r_scenes::circle_button::CircleButton;
use foliage::r_scenes::icon_button::IconButton;
use foliage::r_scenes::icon_text::IconText;
use foliage::r_scenes::text_button::TextButton;
use foliage::scene::Scene;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, Text, TextValue};
use foliage::view::{Aesthetics, Photosynthesis, View};
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
impl<T: Scene> Aesthetic for ButtonDisplay<T> {
    fn pigment(self, aesthetics: &mut Aesthetics) {
        aesthetics.add_scene(
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
            .justify(Justify::Top),
        );
        aesthetics.add_scene(
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
            .justify(Justify::Top),
        );
        aesthetics.add(
            Text::new(
                MaxCharacters(11),
                TextValue::new(self.desc),
                THEME_COLOR::MINUS_THREE,
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
            .justify(Justify::Top),
        );
    }
}
pub struct ButtonShowcase;
impl Photosynthesis for ButtonShowcase {
    const GRID: MacroGrid = MacroGrid::new(8, 6);
    type Chlorophyll = ();

    fn photosynthesize(cmd: &mut Commands, _res: &mut SystemParamItem<Self::Chlorophyll>) -> View {
        let mut aesthetics = Aesthetics::new(cmd);
        aesthetics.add_scene(
            IconText::new(
                FeatherIcon::Menu,
                Color::GREY,
                MaxCharacters(11),
                TextValue::new("button.rs"),
                Color::GREY,
            ),
            ResponsiveSegment::base(Segment::new(
                3.near().to(6.far()),
                1.near().to(1.far()).maximum(60.0),
            ))
            .justify(Justify::Top),
        );
        ButtonDisplay::new(
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::MINUS_THREE,
                Color::BLACK,
            ),
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::ring(),
                THEME_COLOR::MINUS_THREE,
                Color::BLACK,
            ),
            "base".to_string(),
            2,
            None,
            Some(45.0),
        )
        .pigment(&mut aesthetics);
        ButtonDisplay::new(
            TextButton::new(
                TextValue::new("copy"),
                MaxCharacters(4),
                ElementStyle::fill(),
                THEME_COLOR::MINUS_ONE,
                Color::BLACK,
            ),
            TextButton::new(
                TextValue::new("copy"),
                MaxCharacters(4),
                ElementStyle::ring(),
                THEME_COLOR::MINUS_ONE,
                Color::BLACK,
            ),
            "text".to_string(),
            3,
            None,
            Some(45.0),
        )
        .pigment(&mut aesthetics);
        ButtonDisplay::new(
            CircleButton::new(
                FeatherIcon::Copy,
                ElementStyle::fill(),
                THEME_COLOR::PLUS_ONE,
                Color::BLACK,
            ),
            CircleButton::new(
                FeatherIcon::Copy,
                ElementStyle::ring(),
                THEME_COLOR::PLUS_ONE,
                Color::BLACK,
            ),
            "circle".to_string(),
            4,
            Some(55.0),
            Some(55.0),
        )
        .pigment(&mut aesthetics);
        ButtonDisplay::new(
            IconButton::new(
                FeatherIcon::Copy,
                ElementStyle::fill(),
                THEME_COLOR::PLUS_THREE,
                Color::BLACK,
            ),
            IconButton::new(
                FeatherIcon::Copy,
                ElementStyle::ring(),
                THEME_COLOR::PLUS_THREE,
                Color::BLACK,
            ),
            "icon".to_string(),
            5,
            Some(45.0),
            Some(45.0),
        )
        .pigment(&mut aesthetics);
        aesthetics.view()
    }
}