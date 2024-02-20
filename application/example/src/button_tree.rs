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
use foliage::tree::{EntityPool, Responsive, Tree};

pub struct ButtonTree;
impl Tree for ButtonTree {
    const GRID: MacroGrid = MacroGrid::new(4, 4);
    type Resources = ();

    fn plant(cmd: &mut Commands, _res: &mut SystemParamItem<Self::Resources>) {
        let (first, first_bindings) = cmd.responsive_scene(
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
        let branch = cmd.branch(|cmd| {
            let (one, _) = cmd.responsive_scene(
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
            EntityPool::from([one])
            // one will trigger despawns on the bindings so no need to include
        });
        // could use branch entity as value in handler for other button?
        // but dont have elm here would need the same entity elsewhere or different
        // setting mechanism?
        // set branch(entity) trigger.true to run on_enter
        // since defined with resources available in tree,
        // it can copy values to fn closure to not need to pull in resources
        EntityPool::from([first], [(0, branch)])
        // separate branches from elements but both are in pool to despawn
    }
}