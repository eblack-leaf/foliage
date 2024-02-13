use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Commands, IntoSystemConfigs, With};
use foliage_proper::bevy_ecs::query::{Changed, Or, Without};
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::compositor::segment::{
    GapDescriptor, Justify, MacroGrid, Segment, SegmentUnitDesc, SegmentValue,
};
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::icon::{Icon, IconId};
use foliage_proper::rectangle::Rectangle;
use foliage_proper::scene::{Alignment, Binder, Bindings, Scene, SceneComponents};
use foliage_proper::text::{MaxCharacters, Text, TextValue};
use foliage_proper::texture::factors::Progress;

#[derive(Clone)]
pub struct IconText {
    pub icon_id: IconId,
    pub icon_color: Color,
    pub max_chars: MaxCharacters,
    pub text_value: TextValue,
    pub text_color: Color,
}
impl IconText {
    pub fn new<ID: Into<IconId>, C: Into<Color>, TV: Into<TextValue>, MC: Into<MaxCharacters>>(
        id: ID,
        ic: C,
        mc: MC,
        tv: TV,
        tc: C,
    ) -> Self {
        Self {
            icon_id: id.into(),
            icon_color: ic.into(),
            max_chars: mc.into(),
            text_value: tv.into(),
            text_color: tc.into(),
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum IconTextBindings {
    Icon,
    Text,
}
#[derive(Component, Copy, Clone)]
pub struct IconColor(pub Color);
#[derive(Component, Copy, Clone)]
pub struct TextColor(pub Color);
#[derive(Bundle)]
pub struct IconTextComponents {
    pub max_char: MaxCharacters,
    pub text_value: TextValue,
    pub icon_color: IconColor,
    pub text_color: TextColor,
    pub icon_id: IconId,
}
impl IconTextComponents {
    pub fn new<ID: Into<IconId>, C: Into<Color>, TV: Into<TextValue>, MC: Into<MaxCharacters>>(
        mc: MC,
        tv: TV,
        ic: C,
        tc: C,
        id: ID,
    ) -> Self {
        Self {
            max_char: mc.into(),
            text_value: tv.into(),
            icon_color: IconColor(ic.into()),
            text_color: TextColor(tc.into()),
            icon_id: id.into(),
        }
    }
}
#[inner_set_descriptor]
pub enum IconTextConfig {
    Area,
}
impl Leaf for IconText {
    type SetDescriptor = IconTextConfig;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            foliage_proper::scene::config::<IconText>.in_set(Self::SetDescriptor::Area),
        );
    }
}
impl Scene for IconText {
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static MaxCharacters,
                &'static TextColor,
                &'static TextValue,
                &'static IconColor,
                &'static IconId,
            ),
            With<Tag<IconText>>,
        >,
        Query<'static, 'static, &'static mut Color, Without<Tag<IconText>>>,
        Query<
            'static,
            'static,
            (&'static mut TextValue, &'static mut MaxCharacters),
            Without<Tag<IconText>>,
        >,
        Query<'static, 'static, &'static mut IconId, Without<Tag<IconText>>>,
    );
    type Filter = Or<(
        Changed<MaxCharacters>,
        Changed<TextColor>,
        Changed<TextValue>,
        Changed<IconColor>,
        Changed<IconId>,
    )>;
    type Components = IconTextComponents;

    fn config(
        entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        let icon = bindings.get(IconTextBindings::Icon);
        let text = bindings.get(IconTextBindings::Text);
        if let Ok((mc, tc, tv, ic, id)) = ext.0.get(entity) {
            *ext.1.get_mut(icon).unwrap() = ic.0;
            *ext.1.get_mut(text).unwrap() = tc.0;
            *ext.2.get_mut(text).unwrap().0 = tv.clone();
            *ext.2.get_mut(text).unwrap().1 = *mc;
            *ext.3.get_mut(icon).unwrap() = *id;
        }
    }

    fn create(self, cmd: &mut Commands) -> (Entity, SceneComponents<Self::Components>) {
        let mut binder = Binder::new(cmd);
        binder.bind(
            IconTextBindings::Icon,
            Alignment::new(
                Segment::new(
                    1.near().to(1.far()).minimum(20.0).maximum(100.0),
                    0.2.relative()
                        .to(0.8.relative())
                        .minimum(20.0)
                        .maximum(100.0),
                )
                .with_aspect(1.0),
                0,
            ),
            Icon::new(self.icon_id, self.icon_color),
            cmd,
        );
        let factor = 7;
        binder.bind(
            IconTextBindings::Text,
            Alignment::new(
                Segment::new(
                    2.near().to(factor.far()),
                    0.relative().to(1.relative()).minimum(24.0).maximum(100.0),
                )
                .with_aspect(self.max_chars.0 as CoordinateUnit / 2f32),
                0,
            )
            .justify(Justify::Left),
            Text::new(self.max_chars, self.text_value.clone(), self.text_color),
            cmd,
        );
        // binder.bind(
        //     2,
        //     Alignment::new(
        //         Segment::new(
        //             1.near().to(1.far()).minimum(20.0).maximum(100.0),
        //             0.2.relative()
        //                 .to(0.7.relative())
        //                 .minimum(20.0)
        //                 .maximum(100.0),
        //         )
        //         .with_aspect(1.0),
        //         5,
        //     ),
        //     Rectangle::new(Area::default(), Color::WHITE, Progress::full()),
        //     cmd,
        // );
        // binder.bind(
        //     3,
        //     Alignment::new(
        //         Segment::new(
        //             2.near().to(factor.far()),
        //             0.relative().to(1.relative()).minimum(24.0).maximum(100.0),
        //         )
        //         .with_aspect(self.max_chars.0 as CoordinateUnit / 2f32),
        //         5,
        //     ),
        //     Rectangle::new(Area::default(), Color::WHITE, Progress::full()),
        //     cmd,
        // );
        (
            binder.root(),
            SceneComponents::new(
                MacroGrid::new(factor as SegmentValue, 4).assign_gap(GapDescriptor::Both, 4.0),
                binder.bindings(),
                IconTextComponents::new(
                    self.max_chars,
                    self.text_value,
                    self.icon_color,
                    self.text_color,
                    self.icon_id,
                ),
            ),
        )
    }
}
