use crate::icon_text::{IconColor, IconText, TextColor};
use foliage_macros::InnerSceneBinding;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::change_detection::Res;
use foliage_proper::bevy_ecs::prelude::{Changed, Commands, Component, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Or, With, Without};
use foliage_proper::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::InterfaceContext;
use foliage_proper::differential::Despawn;
use foliage_proper::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::icon::{Icon, IconId};
use foliage_proper::interaction::InteractionListener;
use foliage_proper::panel::{Panel, PanelStyle};
use foliage_proper::scene::align::{SceneAligner, SceneAlignment};
use foliage_proper::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage_proper::set_descriptor;
use foliage_proper::text::font::MonospacedFont;
use foliage_proper::text::{MaxCharacters, TextValue};
use foliage_proper::window::ScaleFactor;

#[derive(Bundle)]
pub struct ButtonComponents {
    tag: Tag<ButtonComponents>,
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
        elm.main().add_systems((
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
            With<Tag<ButtonComponents>>,
        ),
    >,
    mut area_query: Query<&mut Area<InterfaceContext>, Without<Tag<ButtonComponents>>>,
    mut max_characters_query: Query<&mut MaxCharacters, Without<Tag<ButtonComponents>>>,
    mut colors: Query<(&mut IconColor, &mut TextColor)>,
    mut panel_styles: Query<(&mut PanelStyle, &mut Color), Without<Tag<ButtonComponents>>>,
    mut coordinator: ResMut<SceneCoordinator>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
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
            let s = ButtonComponents::fill_status(state);
            *style = s;
            *color = foreground_color.0;
        }
        if let Ok(mut content_area) = area_query.get_mut(panel_node) {
            *content_area = *button_area;
        }
        let icon_text_node = coordinator.binding_entity(&icon_text_ac);
        if let Ok((mut ic, mut tc)) = colors.get_mut(icon_text_node) {
            let color = match state {
                ButtonStyle::Ring => foreground_color.0,
                ButtonStyle::Fill => background_color.0,
            };
            ic.0 = color;
            tc.0 = color;
        }
        if let Ok(mut area) = area_query.get_mut(icon_text_node) {
            let ita = *button_area * (0.8, 0.8).into();
            let metrics = crate::icon_text::metrics(ita, max_char, &font, &scale_factor);
            *area = metrics.2;
        }
        if let Ok(mut max_characters) = max_characters_query.get_mut(icon_text_node) {
            *max_characters = *max_char;
        }
    }
}
#[derive(Clone)]
pub struct Button {
    pub style: ButtonStyle,
    pub text: TextValue,
    pub max_char: MaxCharacters,
    pub icon_id: IconId,
    pub foreground_color: Color,
    pub background_color: Color,
}
impl Button {
    pub fn new<ID: Into<IconId>, C: Into<Color>>(
        style: ButtonStyle,
        text: TextValue,
        max_char: MaxCharacters,
        icon_id: ID,
        foreground_color: C,
        background_color: C,
    ) -> Self {
        Self {
            style,
            text,
            max_char,
            icon_id: icon_id.into(),
            foreground_color: foreground_color.into(),
            background_color: background_color.into(),
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum ButtonBindings {
    Panel,
    IconText,
}
impl Scene for Button {
    type Bindings = ButtonBindings;
    type Components = ButtonComponents;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);
    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: Self,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder,
    ) -> Self::Components {
        cmd.entity(binder.this())
            .insert(InteractionListener::default());
        let fill = Self::Components::fill_status(&args.style);
        binder.bind(
            ButtonBindings::Panel,
            (0.close(), 0.close(), 1),
            Panel::new(fill, anchor.0.section.area, args.foreground_color),
            cmd,
        );
        binder.bind_scene(
            ButtonBindings::IconText.into(),
            SceneAlignment::from((0.center(), 0.center(), 0)),
            anchor.0.section.area * (0.8, 0.8).into(),
            IconText::new(
                args.icon_id,
                args.max_char,
                args.text.clone(),
                args.foreground_color,
                args.foreground_color,
            ),
            external_args,
            cmd,
        );
        Self::Components {
            tag: Tag::<Self::Components>::new(),
            foreground_color: ForegroundColor(args.foreground_color),
            background_color: BackgroundColor(args.background_color),
            max_characters: args.max_char,
            button_style: args.style,
            base_style: BaseStyle(args.style),
        }
    }
}

impl ButtonComponents {
    fn fill_status(style: &ButtonStyle) -> PanelStyle {
        match style {
            ButtonStyle::Ring => PanelStyle::ring(),
            ButtonStyle::Fill => PanelStyle::fill(),
        }
    }
}