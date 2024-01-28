use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::icon::{Icon, IconId, IconScale};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use crate::text::font::MonospacedFont;
use crate::text::{FontSize, MaxCharacters, Text, TextValue};
use crate::window::ScaleFactor;
use bevy_ecs::change_detection::Res;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};

#[derive(Bundle)]
pub struct IconText {
    tag: Tag<Self>,
    id: IconId,
    max_chars: MaxCharacters,
    text_value: TextValue,
    icon_color: IconColor,
    text_color: TextColor,
}
#[derive(Copy, Clone, Component, Default)]
pub struct IconColor(pub Color);
#[derive(Copy, Clone, Component, Default)]
pub struct TextColor(pub Color);
pub enum IconTextBindings {
    Icon,
    Text,
}
impl From<IconTextBindings> for SceneBinding {
    fn from(value: IconTextBindings) -> Self {
        Self(value as i32)
    }
}
pub struct IconTextArgs {
    id: IconId,
    max_chars: MaxCharacters,
    text_value: TextValue,
    icon_color: Color,
    text_color: Color,
}
impl IconTextArgs {
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
            resize
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
    let (fs, fa) = font.best_fit(*max_characters, area * (0.6, 0.9).into(), &scale_factor);
    let icon_scale = IconScale::from_dim((fa.height * 0.8).min(fa.width * 0.3));
    let spacing = (icon_scale.px() - area.height).abs() / 2f32;
    let text_offset = icon_scale.px() + spacing * 2f32;
    let total = icon_scale.px() + spacing + fa.width;
    let half = total / 2f32;
    let current_center = spacing + half;
    let center_threshold = area.width * 0.5;
    let adjustment = if current_center < center_threshold {
        center_threshold - current_center
    } else {
        0f32
    };
    (
        icon_scale,
        fs,
        spacing + adjustment,
        text_offset + adjustment,
    )
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
            With<Tag<IconText>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut texts: Query<(&mut MaxCharacters, &mut TextValue, &mut FontSize), Without<Tag<IconText>>>,
    mut icons: Query<(&mut IconId, &mut Area<InterfaceContext>), Without<Tag<IconText>>>,
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
        coordinator.get_alignment_mut(&icon_ac).pos.horizontal = iap.near();
        let icon_entity = coordinator.binding_entity(&icon_ac);
        let text_ac = handle.access_chain().target(IconTextBindings::Text);
        coordinator.get_alignment_mut(&text_ac).pos.horizontal = tap.near();
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
    type Args<'a> = IconTextArgs;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        let (is, fs, iap, tap) = metrics(
            anchor.0.section.area,
            &args.max_chars,
            &external_args.0,
            &external_args.1,
        );
        binder.bind(
            Self::Bindings::Icon,
            (iap.near(), 0.center(), 0),
            Icon::new(args.id, is, args.icon_color),
            cmd,
        );
        binder.bind(
            Self::Bindings::Text,
            (tap.near(), 0.center(), 0),
            Text::new(args.max_chars, fs, args.text_value.clone(), args.text_color),
            cmd,
        );
        Self {
            tag: Tag::new(),
            id: args.id,
            max_chars: args.max_chars,
            text_value: args.text_value.clone(),

            icon_color: IconColor(args.icon_color),
            text_color: TextColor(args.text_color),
        }
    }
}
