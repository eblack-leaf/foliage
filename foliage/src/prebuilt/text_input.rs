use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::{CoordinateUnit, DeviceContext, InterfaceContext};
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
use bevy_ecs::change_detection::Mut;
use bevy_ecs::component::Component;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Query, Res, ResMut, SystemParamItem};
use winit::keyboard::NamedKey;

#[derive(Bundle)]
pub struct TextInput {
    tag: Tag<Self>,
    text: TextValue,
    foreground: ForegroundColor,
    background: BackgroundColor,
    hint_text: HintText,
    cursor_offset: CursorOffset,
    dims: CursorDims,
    max_chars: MaxCharacters,
}
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
                args.foreground.with_alpha(0.5),
                Progress::full(),
            ),
            cmd,
        );
        binder.bind(
            TextInputBindings::Text,
            (SPACING.near(), 0.center(), 0),
            Text::new(args.max_chars, fs, args.text.clone(), args.foreground),
            cmd,
        );
        Self {
            tag: Tag::new(),
            text: args.text.clone(),
            foreground: ForegroundColor(args.foreground),
            background: BackgroundColor(args.background),
            hint_text: HintText(args.hint_text.clone().unwrap_or_default()),
            cursor_offset: CursorOffset(args.text.0.len().min(args.max_chars.0 as usize) as u32),
            dims: CursorDims(character_dims.to_interface(external_args.1.factor())),
            max_chars: args.max_chars,
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
}
impl TextInputArgs {
    pub fn new<C: Into<Color>>(
        max_characters: MaxCharacters,
        text: TextValue,
        hint_text: Option<TextValue>,
        foreground: C,
        bg: C,
    ) -> Self {
        Self {
            max_chars: max_characters,
            foreground: foreground.into(),
            background: bg.into(),
            hint_text,
            text,
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
) {
    for (handle, area, despawn, fc, bc, mc, mut dims) in scenes.iter_mut() {
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
        *colors.get_mut(cursor).unwrap() = fc.0;
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
fn cursor_on_click(
    mut text_inputs: Query<
        (
            &Position<InterfaceContext>,
            &SceneHandle,
            &Despawn,
            &mut CursorOffset,
            &InteractionListener,
            &CursorDims,
            &MaxCharacters,
            &TextValue,
        ),
        (Changed<InteractionListener>, With<Tag<TextInput>>),
    >,
    virtual_keyboard: Res<VirtualKeyboardAdapter>,
    // mut ies: EventReader<InteractionEvent>,
    // primary_interaction: Res<PrimaryInteraction>,
) {
    for (pos, handle, despawn, mut offset, listener, dims, mc, text_val) in text_inputs.iter_mut() {
        if despawn.should_despawn() {
            continue;
        }
        if listener.active() {
            offset.0 = (((listener.interaction.current.x - pos.x - SPACING) / dims.0.width).floor()
                as u32)
                .min(mc.0.checked_sub(1).unwrap_or_default())
                .min(text_val.0.len().checked_sub(1).unwrap_or_default() as u32);
            virtual_keyboard.open(VirtualKeyboardType::Keyboard);
        }
    }
}
fn update_cursor_alignment(
    text_inputs: Query<
        (
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
) {
    for (handle, offset, despawn, dims, bg_color) in text_inputs.iter() {
        if despawn.should_despawn() {
            continue;
        }
        let cursor = handle.access_chain().target(TextInputBindings::Cursor);
        let text =
            coordinator.binding_entity(&handle.access_chain().target(TextInputBindings::Text));
        color_changes.get_mut(text).unwrap().0.clear();
        color_changes
            .get_mut(text)
            .unwrap()
            .0
            .insert(offset.0 as TextKey, bg_color.0);
        coordinator.get_alignment_mut(&cursor).pos.horizontal =
            (dims.0.width * offset.0 as f32 + SPACING).near();
    }
}
fn handle_input(
    mut text_inputs: Query<
        (
            &SceneHandle,
            &mut TextValue,
            &HintText,
            &mut CursorOffset,
            &MaxCharacters,
            &Despawn,
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
            if let Ok((handle, mut text_val, hint, mut offset, max_chars, despawn)) =
                text_inputs.get_mut(focused)
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
                                    bounded_offset(&mut offset, i, *max_chars, &text_val);
                                }
                            }
                            NamedKey::ArrowRight => {
                                if e.state.is_pressed() {
                                    let i = offset.0.checked_add(1).unwrap_or_default();
                                    bounded_offset(&mut offset, i, *max_chars, &text_val);
                                }
                            }
                            NamedKey::Backspace => {
                                // if pressed start slowly deleting
                                // if released stop deleting
                                if e.state.is_pressed() {
                                    if !text_val.0.is_empty() {
                                        if let Some(u) = offset.0.checked_sub(1) {
                                            if text_val.0.chars().nth(u as usize).is_some() {
                                                text_val.0.remove(u as usize);
                                            }
                                            bounded_offset(&mut offset, u, *max_chars, &text_val);
                                        }
                                    }
                                }
                            }
                            NamedKey::Delete => {
                                if e.state.is_pressed() {
                                    if !text_val.0.is_empty() {
                                        if text_val.0.chars().nth(offset.0 as usize).is_some() {
                                            text_val.0.remove(offset.0 as usize);
                                        }
                                    }
                                }
                            }
                            NamedKey::Space => {
                                if e.state.is_pressed() {
                                    let t = nk.to_text().unwrap();
                                    add_text_input(&mut text_val, &mut offset, max_chars, t);
                                }
                            }
                            _ => {}
                        }
                    }
                    Key::Character(ch) => {
                        if e.state.is_pressed() {
                            add_text_input(&mut text_val, &mut offset, max_chars, ch.as_str());
                        }
                    }
                    Key::Unidentified(_) => {}
                    Key::Dead(_) => {}
                }
                let text_entity = coordinator
                    .binding_entity(&handle.access_chain().target(TextInputBindings::Text));
                if text_val.0.is_empty() {
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
    text_val: &TextValue,
) {
    offset.0 = new
        .min(max_chars.0.checked_sub(1).unwrap_or_default())
        .min(text_val.0.len() as u32);
}
fn add_text_input(
    mut text_val: &mut Mut<TextValue>,
    mut offset: &mut Mut<CursorOffset>,
    max_chars: &MaxCharacters,
    t: &str,
) {
    if text_val.0.len() + t.len() < max_chars.0 as usize {
        for (i, c) in t.chars().enumerate() {
            text_val.0.insert(offset.0 as usize + i, c);
            let index = offset.0 + 1;
            bounded_offset(offset, index, *max_chars, &text_val);
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
