use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::interaction::{FocusedEntity, InteractionListener, Key, KeyboardEvent};
use crate::panel::{Panel, PanelStyle};
use crate::prebuilt::button::{BackgroundColor, ForegroundColor};
use crate::rectangle::Rectangle;
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use crate::text::font::MonospacedFont;
use crate::text::{FontSize, GlyphColorChanges, MaxCharacters, Text, TextKey, TextValue};
use crate::texture::factors::Progress;
use crate::virtual_keyboard::{VirtualKeyboardAdapter, VirtualKeyboardType};
use crate::window::ScaleFactor;
use bevy_ecs::component::Component;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Bundle, Commands, DetectChanges, Entity, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Query, Res, ResMut, SystemParamItem};
use compact_str::CompactString;
use winit::keyboard::NamedKey;
#[derive(Component, Clone, Default)]
pub struct ActualText(pub CompactString);
#[derive(Bundle)]
pub struct TextInput {
    tag: Tag<Self>,
    text: TextValue,
    actual_text: ActualText,
    foreground: ForegroundColor,
    background: BackgroundColor,
    hint_text: HintText,
    cursor_offset: CursorOffset,
    dims: CursorDims,
    max_chars: MaxCharacters,
    is_password: IsPassword,
}
#[derive(Component, Copy, Clone, Default)]
pub struct IsPassword(pub bool);
const SPACING: CoordinateUnit = 4.0;
#[derive(Component, Copy, Clone)]
pub(crate) struct CursorDims(pub(crate) Area<InterfaceContext>);
impl Scene for TextInput {
    type Bindings = TextInputBindings;
    type Args<'a> = TextInputArgs;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        let (fs, _fa) = external_args.0.best_fit(
            args.max_chars,
            anchor.0.section.area * (0.9, 0.9).into(),
            &external_args.1,
        );
        let character_dims = external_args
            .0
            .character_dimensions(fs.px(external_args.1.factor()));
        cmd.entity(binder.this())
            .insert(InteractionListener::default());
        binder.bind(
            TextInputBindings::Panel,
            (0.near(), 0.near(), 2),
            Panel::new(PanelStyle::fill(), anchor.0.section.area, args.background),
            cmd,
        );
        binder.bind(
            TextInputBindings::Cursor,
            (SPACING.near(), 0.center(), 1),
            Rectangle::new(
                character_dims.to_numerical().as_interface(),
                args.foreground.with_alpha(0.0),
                Progress::full(),
            ),
            cmd,
        );
        let display_text = if args.is_password {
            if args.text.0.is_empty() {
                args.hint_text.clone().unwrap_or_default()
            } else {
                TextValue::new(
                    std::iter::repeat("*")
                        .take(args.text.0.len())
                        .collect::<String>(),
                )
            }
        } else {
            if args.text.0.is_empty() {
                args.hint_text.clone().unwrap_or_default()
            } else {
                args.text.clone()
            }
        };
        binder.bind(
            TextInputBindings::Text,
            (SPACING.near(), 0.center(), 0),
            Text::new(args.max_chars, fs, display_text.clone(), args.foreground),
            cmd,
        );
        Self {
            tag: Tag::new(),
            text: display_text,
            actual_text: ActualText(args.text.0.clone()),
            foreground: ForegroundColor(args.foreground),
            background: BackgroundColor(args.background),
            hint_text: HintText(args.hint_text.clone().unwrap_or_default()),
            cursor_offset: CursorOffset(args.text.0.len().min(args.max_chars.0 as usize) as u32),
            dims: CursorDims(character_dims.to_interface(external_args.1.factor())),
            max_chars: args.max_chars,
            is_password: IsPassword(args.is_password),
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct CursorOffset(pub u32);
pub struct TextInputArgs {
    max_chars: MaxCharacters,
    foreground: Color,
    background: Color,
    hint_text: Option<TextValue>,
    text: TextValue,
    is_password: bool,
}
impl TextInputArgs {
    pub fn new<C: Into<Color>>(
        max_characters: MaxCharacters,
        text: TextValue,
        hint_text: Option<TextValue>,
        foreground: C,
        bg: C,
        is_password: bool,
    ) -> Self {
        Self {
            max_chars: max_characters,
            foreground: foreground.into(),
            background: bg.into(),
            hint_text,
            text,
            is_password,
        }
    }
}
pub enum TextInputBindings {
    Cursor,
    Panel,
    Text,
}
impl From<TextInputBindings> for SceneBinding {
    fn from(value: TextInputBindings) -> Self {
        Self(value as i32)
    }
}
fn resize(
    mut scenes: Query<
        (
            Entity,
            &SceneHandle,
            &Area<InterfaceContext>,
            &Despawn,
            &ForegroundColor,
            &BackgroundColor,
            &MaxCharacters,
            &mut CursorDims,
        ),
        (
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<ForegroundColor>,
                Changed<BackgroundColor>,
                Changed<MaxCharacters>,
            )>,
            With<Tag<TextInput>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut texts: Query<(&mut FontSize, &mut MaxCharacters), Without<Tag<TextInput>>>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
    mut areas: Query<&mut Area<InterfaceContext>, Without<Tag<TextInput>>>,
    mut colors: Query<&mut Color>,
    focused_entity: Res<FocusedEntity>,
) {
    for (entity, handle, area, despawn, fc, bc, mc, mut dims) in scenes.iter_mut() {
        if despawn.should_despawn() {
            continue;
        }
        coordinator.update_anchor_area(*handle, *area);
        let panel =
            coordinator.binding_entity(&handle.access_chain().target(TextInputBindings::Panel));
        *areas.get_mut(panel).unwrap() = *area;
        *colors.get_mut(panel).unwrap() = bc.0;
        let cursor =
            coordinator.binding_entity(&handle.access_chain().target(TextInputBindings::Cursor));
        let alpha = if let Some(fe) = focused_entity.0 {
            if fe == entity {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        *colors.get_mut(cursor).unwrap() = fc.0.with_alpha(alpha);
        let (fs, _fa) = font.best_fit(*mc, *area * (0.95, 0.9).into(), &scale_factor);
        let character_dims = font.character_dimensions(fs.px(scale_factor.factor()));
        dims.0 = character_dims.to_interface(scale_factor.factor()) - (0, SPACING).into();
        *areas.get_mut(cursor).unwrap() = dims.0;
        let text_entity =
            coordinator.binding_entity(&handle.access_chain().target(TextInputBindings::Text));
        *texts.get_mut(text_entity).unwrap().0 = fs;
        *texts.get_mut(text_entity).unwrap().1 = *mc;
        *colors.get_mut(text_entity).unwrap() = fc.0;
    }
}
fn clear_cursor(
    mut scenes: Query<
        (
            Entity,
            &SceneHandle,
            &Despawn,
            &ActualText,
            &HintText,
            &mut TextValue,
        ),
        With<Tag<TextInput>>,
    >,
    focused_entity: Res<FocusedEntity>,
    mut colors: Query<&mut Color>,
    coordinator: Res<SceneCoordinator>,
    mut color_changes: Query<&mut GlyphColorChanges>,
    mut texts: Query<&mut TextValue, Without<Tag<TextInput>>>,
) {
    for (entity, handle, despawn, actual, hint, mut text_val) in scenes.iter_mut() {
        if despawn.should_despawn() {
            continue;
        }
        if focused_entity.is_changed() {
            let mut changed = false;
            if let Some(fe) = focused_entity.0 {
                if fe != entity {
                    changed = true;
                }
            } else {
                changed = true;
            }
            if changed {
                let ent = coordinator
                    .binding_entity(&handle.access_chain().target(TextInputBindings::Cursor));
                colors.get_mut(ent).unwrap().alpha = 0.0;
                let ent = coordinator
                    .binding_entity(&handle.access_chain().target(TextInputBindings::Text));
                if actual.0.is_empty() {
                    text_val.0 = hint.0 .0.clone();
                }
                texts.get_mut(ent).unwrap().0 = text_val.0.clone();
                color_changes.get_mut(ent).unwrap().0.clear();
            }
        }
    }
}
fn cursor_on_click(
    mut text_inputs: Query<
        (
            &Position<InterfaceContext>,
            &ActualText,
            &SceneHandle,
            &Despawn,
            &mut CursorOffset,
            &InteractionListener,
            &CursorDims,
            &MaxCharacters,
            &mut TextValue,
        ),
        (Changed<InteractionListener>, With<Tag<TextInput>>),
    >,
    virtual_keyboard: Res<VirtualKeyboardAdapter>,
    mut colors: Query<&mut Color>,
    coordinator: Res<SceneCoordinator>,
    mut texts: Query<&mut TextValue, Without<Tag<TextInput>>>,
) {
    for (pos, actual, handle, despawn, mut offset, listener, dims, mc, mut text_val) in
        text_inputs.iter_mut()
    {
        if despawn.should_despawn() {
            continue;
        }
        if listener.active() {
            if actual.0.is_empty() {
                let ent = coordinator
                    .binding_entity(&handle.access_chain().target(TextInputBindings::Text));
                text_val.0.clear();
                texts.get_mut(ent).unwrap().0.clear();
            }
            let ent = coordinator
                .binding_entity(&handle.access_chain().target(TextInputBindings::Cursor));
            colors.get_mut(ent).unwrap().alpha = 1.0;
            offset.0 = (((listener.interaction.current.x - pos.x - SPACING) / dims.0.width).floor()
                as u32)
                .min(mc.0.checked_sub(1).unwrap_or_default())
                .min(text_val.0.len() as u32);
            virtual_keyboard.open(VirtualKeyboardType::Keyboard);
        }
    }
}
fn update_cursor_alignment(
    text_inputs: Query<
        (
            Entity,
            &SceneHandle,
            &CursorOffset,
            &Despawn,
            &CursorDims,
            &BackgroundColor,
        ),
        (
            Or<(Changed<CursorOffset>, Changed<CursorDims>)>,
            With<Tag<TextInput>>,
        ),
    >,
    mut color_changes: Query<&mut GlyphColorChanges>,
    mut coordinator: ResMut<SceneCoordinator>,
    focused_entity: Res<FocusedEntity>,
) {
    for (entity, handle, offset, despawn, dims, bg_color) in text_inputs.iter() {
        if despawn.should_despawn() {
            continue;
        }
        let cursor = handle.access_chain().target(TextInputBindings::Cursor);
        let text =
            coordinator.binding_entity(&handle.access_chain().target(TextInputBindings::Text));
        if let Some(fe) = focused_entity.0 {
            if fe == entity {
                color_changes.get_mut(text).unwrap().0.clear();
                color_changes
                    .get_mut(text)
                    .unwrap()
                    .0
                    .insert(offset.0 as TextKey, bg_color.0);
            }
        }
        coordinator.get_alignment_mut(&cursor).pos.horizontal =
            (dims.0.width * offset.0 as f32 + SPACING).near();
    }
}
fn handle_input(
    mut text_inputs: Query<
        (
            Entity,
            &SceneHandle,
            &mut ActualText,
            &mut TextValue,
            &HintText,
            &mut CursorOffset,
            &MaxCharacters,
            &Despawn,
            &IsPassword,
        ),
        With<Tag<TextInput>>,
    >,
    mut texts: Query<&mut TextValue, Without<Tag<TextInput>>>,
    focused_entity: Res<FocusedEntity>,
    mut events: EventReader<KeyboardEvent>,
    coordinator: Res<SceneCoordinator>,
) {
    for e in events.read() {
        if let Some(focused) = focused_entity.0 {
            if let Ok((
                entity,
                handle,
                mut actual,
                mut text_val,
                hint,
                mut offset,
                max_chars,
                despawn,
                is_password,
            )) = text_inputs.get_mut(focused)
            {
                if despawn.should_despawn() {
                    continue;
                }
                match &e.key {
                    Key::Named(nk) => {
                        match nk {
                            NamedKey::ArrowLeft => {
                                if e.state.is_pressed() {
                                    let i = offset.0.checked_sub(1).unwrap_or_default();
                                    bounded_offset(&mut offset, i, *max_chars, &actual.0);
                                }
                            }
                            NamedKey::ArrowRight => {
                                if e.state.is_pressed() {
                                    let i = offset.0.checked_add(1).unwrap_or_default();
                                    bounded_offset(&mut offset, i, *max_chars, &actual.0);
                                }
                            }
                            NamedKey::Backspace => {
                                // if pressed start slowly deleting
                                // if released stop deleting
                                if e.state.is_pressed() {
                                    if !actual.0.is_empty() {
                                        if let Some(u) = offset.0.checked_sub(1) {
                                            if actual.0.chars().nth(u as usize).is_some() {
                                                actual.0.remove(u as usize);
                                                text_val.0.remove(u as usize);
                                            }
                                            bounded_offset(&mut offset, u, *max_chars, &actual.0);
                                        }
                                    }
                                }
                            }
                            NamedKey::Delete => {
                                if e.state.is_pressed() {
                                    if !actual.0.is_empty() {
                                        if actual.0.chars().nth(offset.0 as usize).is_some() {
                                            actual.0.remove(offset.0 as usize);
                                            text_val.0.remove(offset.0 as usize);
                                        }
                                    }
                                }
                            }
                            NamedKey::Space => {
                                if e.state.is_pressed() {
                                    let t = nk.to_text().unwrap();
                                    add_text_input(
                                        &mut actual,
                                        &mut text_val,
                                        offset.as_mut(),
                                        max_chars,
                                        t,
                                        is_password.0,
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                    Key::Character(ch) => {
                        if e.state.is_pressed() {
                            add_text_input(
                                &mut actual,
                                &mut text_val,
                                offset.as_mut(),
                                max_chars,
                                ch.as_str(),
                                is_password.0,
                            );
                        }
                    }
                    Key::Unidentified(_) => {}
                    Key::Dead(_) => {}
                }
                let text_entity = coordinator
                    .binding_entity(&handle.access_chain().target(TextInputBindings::Text));
                if focused_entity.0.is_some() {
                    if focused_entity.0.unwrap() == entity {
                        *texts.get_mut(text_entity).unwrap() = text_val.clone();
                        continue;
                    }
                }
                if actual.0.is_empty() {
                    *texts.get_mut(text_entity).unwrap() = hint.0.clone();
                } else {
                    *texts.get_mut(text_entity).unwrap() = text_val.clone();
                }
            }
        }
    }
}
fn bounded_offset(
    offset: &mut CursorOffset,
    new: u32,
    max_chars: MaxCharacters,
    text_val: &CompactString,
) {
    offset.0 = new
        .min(max_chars.0.checked_sub(1).unwrap_or_default())
        .min(text_val.len() as u32);
}
fn add_text_input(
    actual_text: &mut ActualText,
    text_val: &mut TextValue,
    offset: &mut CursorOffset,
    max_chars: &MaxCharacters,
    t: &str,
    is_password: bool,
) {
    if actual_text.0.len() + t.len() < max_chars.0 as usize {
        for (i, mut c) in t.chars().enumerate() {
            if is_password {
                actual_text.0.insert(offset.0 as usize + i, c);
                c = '*';
            } else {
                actual_text.0.insert(offset.0 as usize + i, c);
            }
            text_val.0.insert(offset.0 as usize + i, c);
            let index = offset.0 + 1;
            bounded_offset(offset, index, *max_chars, &actual_text.0);
        }
    }
}
set_descriptor!(
    pub enum TextInputSets {
        Area,
    }
);
impl Leaf for TextInput {
    type SetDescriptor = TextInputSets;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            (
                resize,
                handle_input,
                cursor_on_click,
                clear_cursor,
                update_cursor_alignment,
            )
                .chain()
                .in_set(Self::SetDescriptor::Area)
                .before(<Text as Leaf>::SetDescriptor::Area)
                .before(<Panel as Leaf>::SetDescriptor::Area),
        );
    }
}
#[derive(Component, Clone, Default)]
pub struct HintText(pub TextValue);
