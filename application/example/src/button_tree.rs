use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::{Commands, Component};
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::color::monochromatic::{AquaMarine as THEME_COLOR, Monochromatic};
use foliage::color::Color;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf};
use foliage::elm::{ElementStyle, Elm};
use foliage::icon::FeatherIcon;
use foliage::layout::Layout;
use foliage::r_scenes::button::Button;
use foliage::r_scenes::icon_text::IconText;
use foliage::scene::ExtendTarget;
use foliage::segment::{Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage::text::{MaxCharacters, Text, TextValue};
use foliage::view::{Aesthetics, BranchHandle, ConditionSet, Photosynthesis, View};
pub struct BranchingButtonShowcase;
impl Photosynthesis for BranchingButtonShowcase {
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
        let desc = aesthetics.add_scene(
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
        aesthetics.extend(desc.root(), SampleHook(true));
        aesthetics.conditional(
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
        let other_desc = aesthetics.conditional_scene(
            1,
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::MINUS_ONE,
                Color::BLACK,
            ),
            ResponsiveSegment::base(Segment::new(
                5.near().to(6.far()).minimum(115.0).maximum(350.0),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    5.near().to(8.far()),
                    2.near().to(2.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
        );
        aesthetics.conditional_extend(other_desc, ExtendTarget::This, OtherHook(true));
        aesthetics.conditional(
            2,
            Text::new(MaxCharacters(11), TextValue::new("base"), THEME_COLOR::BASE),
            ResponsiveSegment::base(Segment::new(
                7.near().to(8.far()),
                2.near().to(2.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
        );
        // tmp
        aesthetics.conditional_scene(
            3,
            Button::new(
                IconText::new(
                    FeatherIcon::Copy,
                    Color::BLACK,
                    MaxCharacters(4),
                    TextValue::new("copy"),
                    Color::BLACK,
                ),
                ElementStyle::fill(),
                THEME_COLOR::MINUS_TWO,
                Color::BLACK,
            ),
            ResponsiveSegment::base(Segment::new(
                2.near().to(3.far()).minimum(115.0).maximum(350.0),
                3.near().to(3.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    1.near().to(4.far()),
                    3.near().to(3.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
        );
        aesthetics.conditional(
            4,
            Text::new(
                MaxCharacters(11),
                TextValue::new("text"),
                THEME_COLOR::PLUS_THREE,
            ),
            ResponsiveSegment::base(Segment::new(
                4.near().to(5.far()),
                3.near().to(3.far()).minimum(30.0).maximum(40.0),
            ))
            .justify(Justify::Top)
            .without_portrait_mobile()
            .without_portrait_tablet(),
        );
        aesthetics.conditional_scene(
            5,
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
                5.near().to(6.far()).minimum(115.0).maximum(350.0),
                3.near().to(3.far()).minimum(30.0).maximum(40.0),
            ))
            .exception(
                [Layout::PORTRAIT_MOBILE],
                Segment::new(
                    5.near().to(8.far()),
                    3.near().to(3.far()).minimum(30.0).maximum(40.0),
                ),
            )
            .justify(Justify::Top),
        );
        aesthetics.view()
    }
}
#[derive(Component)]
pub(crate) struct SampleHook(pub(crate) bool);
#[derive(Component, Clone)]
pub(crate) struct OtherHook(pub(crate) bool);

impl Leaf for BranchingButtonShowcase {
    type SetDescriptor = EmptySetDescriptor;

    fn attach(elm: &mut Elm) {
        elm.add_interaction_handler::<SampleHook, Commands>(|sh, cmd| {
            cmd.spawn(ConditionSet(BranchHandle(0), sh.0));
            cmd.spawn(ConditionSet(BranchHandle(1), sh.0));
            cmd.spawn(ConditionSet(BranchHandle(2), sh.0));
            if !sh.0 {
                cmd.spawn(ConditionSet(BranchHandle(3), false));
                cmd.spawn(ConditionSet(BranchHandle(4), false));
                cmd.spawn(ConditionSet(BranchHandle(5), false));
            }
            sh.0 = !sh.0;
        });
        elm.add_interaction_handler::<OtherHook, Commands>(|sh, cmd| {
            cmd.spawn(ConditionSet(BranchHandle(3), sh.0));
            cmd.spawn(ConditionSet(BranchHandle(4), sh.0));
            cmd.spawn(ConditionSet(BranchHandle(5), sh.0));
            sh.0 = !sh.0;
        });
        elm.enable_conditional::<Text>();
        elm.enable_conditional::<ResponsiveSegment>();
        elm.enable_conditional_scene::<Button>();
        elm.enable_conditional::<OtherHook>();
    }
}
