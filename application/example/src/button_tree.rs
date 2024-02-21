use foliage::bevy_ecs::prelude::Commands;
use foliage::bevy_ecs::system::SystemParamItem;
use foliage::color::monochromatic::{AquaMarine as THEME_COLOR, Monochromatic};
use foliage::color::Color;
use foliage::compositor::layout::Layout;
use foliage::compositor::segment::{
    Justify, MacroGrid, ResponsiveSegment, Segment, SegmentUnitDesc,
};
use foliage::elm::ElementStyle;
use foliage::icon::FeatherIcon;
use foliage::r_scenes::button::Button;
use foliage::r_scenes::icon_text::IconText;
use foliage::text::{MaxCharacters, TextValue};
use foliage::tree::{Seed, Tree, TreeBinder};

pub struct ButtonTree;
impl Seed for ButtonTree {
    const GRID: MacroGrid = MacroGrid::new(4, 4);
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

        binder.tree()
    }
}