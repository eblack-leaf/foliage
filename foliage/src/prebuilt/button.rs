use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::icon::{Icon, IconId};
use crate::interaction::InteractionListener;
use crate::panel::{Panel, PanelStyle};
use crate::prebuilt::icon_text::{IconColor, IconText, IconTextArgs, TextColor};
use crate::scene::align::{SceneAligner, SceneAlignment};
use crate::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use crate::text::font::MonospacedFont;
use crate::text::{MaxCharacters, TextValue};
use crate::window::ScaleFactor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::prelude::{Changed, Commands, Component, IntoSystemConfigs};
use bevy_ecs::query::{Or, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_macros::SceneBinding;

#[derive(Bundle)]
pub struct Button {
    tag: Tag<Button>,
    foreground_color: ForegroundColor,
    background_color: BackgroundColor,
    max_characters: MaxCharacters,
    button_style: ButtonStyle,
    base_style: BaseStyle,
}
#[derive(Component, Copy, Clone)]
pub struct BaseStyle(pub ButtonStyle);
#[derive(Component, Copy, Clone)]
pub enum ButtonStyle {
    Ring = 0,
    Fill,
}
set_descriptor!(
    pub enum SetDescriptors {
        Area,
    }
);
impl Leaf for Button {
    type SetDescriptor = SetDescriptors;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, SetDescriptors::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.job.main().add_systems((
            updates
                .in_set(SetDescriptors::Area)
                .before(<IconText as Leaf>::SetDescriptor::Area)
                .before(<Panel as Leaf>::SetDescriptor::Area)
                .before(<Icon as Leaf>::SetDescriptor::Area),
            interaction_color
                .after(CoreSet::Interaction)
                .before(Self::SetDescriptor::Area),
        ));
    }
}
#[derive(Copy, Clone, Component)]
pub struct ForegroundColor(pub Color);
#[derive(Copy, Clone, Component)]
pub struct BackgroundColor(pub Color);
fn interaction_color(
    mut buttons: Query<
        (&InteractionListener, &mut ButtonStyle, &BaseStyle),
        Changed<InteractionListener>,
    >,
) {
    for (listener, mut style, base) in buttons.iter_mut() {
        if listener.engaged_start() {
            match base.0 {
                ButtonStyle::Ring => {
                    *style = ButtonStyle::Fill;
                }
                ButtonStyle::Fill => {
                    *style = ButtonStyle::Ring;
                }
            }
        } else if listener.engaged_end() {
            *style = base.0;
        }
    }
}
fn updates(
    query: Query<
        (
            &SceneHandle,
            &Area<InterfaceContext>,
            &MaxCharacters,
            &ForegroundColor,
            &BackgroundColor,
            &ButtonStyle,
            &Despawn,
        ),
        (
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<MaxCharacters>,
                Changed<ForegroundColor>,
                Changed<BackgroundColor>,
                Changed<ButtonStyle>,
            )>,
            With<Tag<Button>>,
        ),
    >,
    mut area_query: Query<&mut Area<InterfaceContext>, Without<Tag<Button>>>,
    mut max_characters_query: Query<&mut MaxCharacters, Without<Tag<Button>>>,
    mut colors: Query<(&mut IconColor, &mut TextColor)>,
    mut panel_styles: Query<(&mut PanelStyle, &mut Color), Without<Tag<Button>>>,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    for (handle, button_area, max_char, foreground_color, background_color, state, despawn) in
        query.iter()
    {
        if despawn.should_despawn() {
            continue;
        }
        let panel_ac = handle.access_chain().target(ButtonBindings::Panel);
        let icon_text_ac = handle.access_chain().target(ButtonBindings::IconText);
        coordinator.update_anchor_area(*handle, *button_area);
        let panel_node = coordinator.binding_entity(&panel_ac);
        if let Ok((mut style, mut color)) = panel_styles.get_mut(panel_node) {
            let s = Button::fill_status(state);
            *style = s;
            *color = foreground_color.0;
        }
        if let Ok(mut content_area) = area_query.get_mut(panel_node) {
            *content_area = *button_area;
        }
        let text_node = coordinator.binding_entity(&icon_text_ac);
        if let Ok((mut ic, mut tc)) = colors.get_mut(text_node) {
            let color = match state {
                ButtonStyle::Ring => foreground_color.0,
                ButtonStyle::Fill => background_color.0,
            };
            ic.0 = color;
            tc.0 = color;
        }
        if let Ok(mut area) = area_query.get_mut(text_node) {
            *area = *button_area * (0.9, 0.9).into();
        }
        if let Ok(mut max_characters) = max_characters_query.get_mut(text_node) {
            *max_characters = *max_char;
        }
    }
}
pub struct ButtonArgs {
    pub style: ButtonStyle,
    pub text: TextValue,
    pub max_char: MaxCharacters,
    pub icon_id: IconId,
    pub foreground_color: Color,
    pub background_color: Color,
}
impl ButtonArgs {
    pub fn new<C: Into<Color>>(
        style: ButtonStyle,
        text: TextValue,
        max_char: MaxCharacters,
        icon_id: IconId,
        foreground_color: C,
        background_color: C,
    ) -> Self {
        Self {
            style,
            text,
            max_char,
            icon_id,
            foreground_color: foreground_color.into(),
            background_color: background_color.into(),
        }
    }
}
#[derive(SceneBinding)]
pub enum ButtonBindings {
    Panel,
    IconText,
}
impl Scene for Button {
    type Bindings = ButtonBindings;
    type Args<'a> = ButtonArgs;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);
    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder,
    ) -> Self {
        cmd.entity(binder.this())
            .insert(InteractionListener::default());
        let fill = Self::fill_status(&args.style);
        binder.bind(
            ButtonBindings::Panel,
            (0.near(), 0.near(), 1),
            Panel::new(fill, anchor.0.section.area, args.foreground_color),
            cmd,
        );
        binder.bind_scene::<IconText>(
            ButtonBindings::IconText.into(),
            SceneAlignment::from((0.center(), 0.center(), 0)),
            anchor.0.section.area * (0.8, 0.9).into(),
            &IconTextArgs::new(
                args.icon_id,
                args.max_char,
                args.text.clone(),
                args.foreground_color,
                args.foreground_color,
            ),
            &external_args,
            cmd,
        );
        Self {
            tag: Tag::<Self>::new(),
            foreground_color: ForegroundColor(args.foreground_color),
            background_color: BackgroundColor(args.background_color),
            max_characters: args.max_char,
            button_style: args.style,
            base_style: BaseStyle(args.style),
        }
    }
}

impl Button {
    fn fill_status(style: &ButtonStyle) -> PanelStyle {
        match style {
            ButtonStyle::Ring => PanelStyle::ring(),
            ButtonStyle::Fill => PanelStyle::fill(),
        }
    }
}