use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Commands, Component};
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::color::monochromatic::{AquaMarine as THEME_COLOR, Monochromatic};
use foliage::color::Color;
use foliage::elm::ElementStyle;
use foliage::icon::FeatherIcon;
use foliage::layout::Layout;
use foliage::r_scenes::button::Button;
use foliage::r_scenes::icon_text::IconText;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, Text, TextValue};
use foliage::tree::{Seed, Tree, TreeBinder};
pub struct ShowcaseSeed;
impl Seed for ShowcaseSeed {
    const GRID: MacroGrid = MacroGrid::new(8, 6);
    type Resources = ();

    fn plant(cmd: &mut Commands, _res: &mut SystemParamItem<Self::Resources>) -> Tree {
        let mut binder = TreeBinder::new(cmd);
        binder.responsive_scene(
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
        let desc = binder.responsive_scene(
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::BASE,
                Color::BLACK,
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(3.far()).minimum(115.0).maximum(350.0),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    1.near().to(4.far()),
                    2.near().to(2.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
        );
        binder.extend(desc.root(), SampleHook());
        binder.branch(
            0,
            Text::new(
                MaxCharacters(11),
                TextValue::new("base"),
                THEME_COLOR::MINUS_THREE,
            ),
            ResponsiveSegment::base(Segment::new(
                4.near().to(5.far()),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
        );
        binder.tree()
    }
}
#[derive(Component)]
pub(crate) struct SampleHook();
