use foliage_macros::InnerSceneBinding;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::change_detection::Res;
use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, Or, With, Without};
use foliage_proper::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::{CoordinateUnit, InterfaceContext};
use foliage_proper::differential::Despawn;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::icon::{Icon, IconId, IconScale};
use foliage_proper::scene::align::SceneAligner;
use foliage_proper::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage_proper::set_descriptor;
use foliage_proper::text::font::MonospacedFont;
use foliage_proper::text::{FontSize, GlyphColorChanges, MaxCharacters, Text, TextValue};
use foliage_proper::window::ScaleFactor;
#[derive(Bundle)]
pub struct IconTextComponents {
    tag: Tag<Self>,
    id: IconId,
    max_chars: MaxCharacters,
    text_value: TextValue,
    icon_color: IconColor,
    text_color: TextColor,
    color_changes: GlyphColorChanges,
}
#[derive(Copy, Clone, Component, Default)]
pub struct IconColor(pub Color);
#[derive(Copy, Clone, Component, Default)]
pub struct TextColor(pub Color);
#[derive(InnerSceneBinding)]
pub enum IconTextBindings {
    Icon,
    Text,
}
#[derive(Clone)]
pub struct IconText {
    id: IconId,
    max_chars: MaxCharacters,
    text_value: TextValue,
    icon_color: Color,
    text_color: Color,
}
impl IconText {
    pub fn new<ID: Into<IconId>, MC: Into<MaxCharacters>, TV: Into<TextValue>, C: Into<Color>>(
        id: ID,
        mc: MC,
        tv: TV,
        ic: C,
        tc: C,
    ) -> Self {
        Self {
            id: id.into(),
            max_chars: mc.into(),
            text_value: tv.into(),
            icon_color: ic.into(),
            text_color: tc.into(),
        }
    }
}
set_descriptor!(
    pub enum IconTextSets {
        Area,
    }
);
impl Leaf for IconText {
    type SetDescriptor = IconTextSets;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            (resize, color_changes)
                .chain()
                .in_set(Self::SetDescriptor::Area)
                .before(<Text as Leaf>::SetDescriptor::Area)
                .before(<Icon as Leaf>::SetDescriptor::Area),
        );
    }
}
fn metrics(
    area: Area<InterfaceContext>,
    max_characters: &MaxCharacters,
    font: &MonospacedFont,
    scale_factor: &ScaleFactor,
) -> (IconScale, FontSize, CoordinateUnit, CoordinateUnit) {
    let spacing = 8.0;
    let (fs, fa) = font.best_fit(*max_characters, area * (0.75, 1.2).into(), scale_factor);
    let icon_scale = IconScale::from_dim(
        ((area.width - spacing) * 0.25)
            .min(area.height * 0.7)
            .min(fa.height),
    );
    // let center_x = area.width / 2f32;
    // let element_center_x = (icon_scale.px() + spacing + fa.width) / 2f32;
    // let offset = if element_center_x < center_x {
    //     center_x - element_center_x
    // } else {
    //     0.0
    // };
    let offset = 0.0;
    (icon_scale, fs, offset, offset + icon_scale.px() + spacing)
}
fn color_changes(
    mut scenes: Query<
        (&SceneHandle, &GlyphColorChanges, &Despawn),
        (Changed<GlyphColorChanges>, With<Tag<IconTextComponents>>),
    >,
    coordinator: Res<SceneCoordinator>,
    mut color_changes: Query<&mut GlyphColorChanges, Without<Tag<IconTextComponents>>>,
) {
    for (handle, cc, despawn) in scenes.iter_mut() {
        if despawn.should_despawn() {
            continue;
        }
        let entity =
            coordinator.binding_entity(&handle.access_chain().target(IconTextBindings::Text));
        color_changes.get_mut(entity).unwrap().0 = cc.0.clone();
    }
}
fn resize(
    scenes: Query<
        (
            &SceneHandle,
            &Area<InterfaceContext>,
            &MaxCharacters,
            &TextValue,
            &IconId,
            &Despawn,
            &IconColor,
            &TextColor,
        ),
        (
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<IconId>,
                Changed<MaxCharacters>,
                Changed<TextValue>,
                Changed<IconColor>,
                Changed<TextColor>,
            )>,
            With<Tag<IconTextComponents>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut texts: Query<
        (&mut MaxCharacters, &mut TextValue, &mut FontSize),
        Without<Tag<IconTextComponents>>,
    >,
    mut icons: Query<(&mut IconId, &mut Area<InterfaceContext>), Without<Tag<IconTextComponents>>>,
    mut colors: Query<&mut Color>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for (handle, area, max_char, text_val, icon_id, despawn, ic, tc) in scenes.iter() {
        if despawn.should_despawn() {
            continue;
        }
        coordinator.update_anchor_area(*handle, *area);
        let (is, fs, iap, tap) = metrics(*area, max_char, &font, &scale_factor);
        let icon_ac = handle.access_chain().target(IconTextBindings::Icon);
        coordinator.get_alignment_mut(&icon_ac).pos.horizontal = iap.from_left();
        let icon_entity = coordinator.binding_entity(&icon_ac);
        let text_ac = handle.access_chain().target(IconTextBindings::Text);
        coordinator.get_alignment_mut(&text_ac).pos.horizontal = tap.from_left();
        let text_entity = coordinator.binding_entity(&text_ac);
        *texts.get_mut(text_entity).unwrap().0 = *max_char;
        *texts.get_mut(text_entity).unwrap().1 = text_val.clone();
        *texts.get_mut(text_entity).unwrap().2 = fs;
        *icons.get_mut(icon_entity).unwrap().0 = *icon_id;
        icons.get_mut(icon_entity).unwrap().1.width = is.px();
        *colors.get_mut(icon_entity).unwrap() = ic.0;
        *colors.get_mut(text_entity).unwrap() = tc.0;
    }
}
impl Scene for IconText {
    type Bindings = IconTextBindings;
    type Components = IconTextComponents;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: Self,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self::Components {
        let (is, fs, iap, tap) = metrics(
            anchor.0.section.area,
            &args.max_chars,
            &external_args.0,
            &external_args.1,
        );
        binder.bind(
            Self::Bindings::Icon,
            (iap.from_left(), 0.center(), 0),
            Icon::new(args.id, is, args.icon_color),
            cmd,
        );
        binder.bind(
            Self::Bindings::Text,
            (tap.from_left(), 0.center(), 0),
            Text::new(args.max_chars, fs, args.text_value.clone(), args.text_color),
            cmd,
        );
        Self::Components {
            tag: Tag::new(),
            id: args.id,
            max_chars: args.max_chars,
            text_value: args.text_value.clone(),
            icon_color: IconColor(args.icon_color),
            text_color: TextColor(args.text_color),
            color_changes: GlyphColorChanges::default(),
        }
    }
}
