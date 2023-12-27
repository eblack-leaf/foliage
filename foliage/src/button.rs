use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::icon::{Icon, IconId, IconScale};
use crate::panel::{Panel, PanelStyle};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use crate::text::font::MonospacedFont;
use crate::text::{FontSize, MaxCharacters, Text, TextValue};
use crate::window::ScaleFactor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::prelude::{Changed, Commands, Component, IntoSystemConfigs};
use bevy_ecs::query::{Or, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};

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
        Button,
    }
);
impl Leaf for Button {
    type SetDescriptor = SetDescriptors;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, SetDescriptors::Button);
    }

    fn attach(elm: &mut Elm) {
        elm.job.main().add_systems((updates
            .in_set(ExternalSet::Configure)
            .in_set(SetDescriptors::Button)
            .before(<Text as Leaf>::SetDescriptor::Area)
            .before(<Panel as Leaf>::SetDescriptor::Area)
            .before(<Icon as Leaf>::SetDescriptor::Area),));
    }
}
#[derive(Copy, Clone, Component)]
pub struct ForegroundColor(pub Color);
#[derive(Copy, Clone, Component)]
pub struct BackgroundColor(pub Color);
fn updates(
    query: Query<
        (
            &SceneHandle,
            &Area<InterfaceContext>,
            &MaxCharacters,
            &ForegroundColor,
            &BackgroundColor,
            &ButtonStyle,
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
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
    mut scales: Query<&mut IconScale>,
    mut font_sizes: Query<(&mut FontSize, &mut MaxCharacters), Without<Tag<Button>>>,
    mut colors: Query<&mut Color>,
    mut panel_styles: Query<(&mut PanelStyle, &mut Area<InterfaceContext>), Without<Tag<Button>>>,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    for (handle, button_area, max_char, foreground_color, background_color, state) in query.iter() {
        let (fs, text_offset, _text_area, icon_scale, padding) =
            button_metrics(*button_area, *max_char, &font, &scale_factor);
        let panel_ac = handle.access_chain().target(ButtonBindings::Panel);
        let text_ac = handle.access_chain().target(ButtonBindings::Text);
        let icon_ac = handle.access_chain().target(ButtonBindings::Icon);
        coordinator.update_alignment(&text_ac).pos.horizontal = text_offset.near();
        coordinator.update_alignment(&icon_ac).pos.horizontal = padding.far();
        let coordinate = coordinator.anchor(*handle).0.with_area(*button_area);
        coordinator.update_anchor(*handle, coordinate);
        let panel_node = coordinator.binding_entity(&panel_ac);
        if let Ok(mut color) = colors.get_mut(panel_node) {
            *color = match state {
                ButtonStyle::Ring => foreground_color.0,
                ButtonStyle::Fill => foreground_color.0,
            };
        }
        if let Ok((mut style, mut content_area)) = panel_styles.get_mut(panel_node) {
            *style = match state {
                ButtonStyle::Ring => PanelStyle::ring(),
                ButtonStyle::Fill => PanelStyle::fill(),
            };
            *content_area = *button_area;
        }
        let text_node = coordinator.binding_entity(&text_ac);
        if let Ok(mut color) = colors.get_mut(text_node) {
            *color = match state {
                ButtonStyle::Ring => foreground_color.0,
                ButtonStyle::Fill => background_color.0,
            };
        }
        if let Ok((mut font_size, mut max_characters)) = font_sizes.get_mut(text_node) {
            *font_size = fs;
            *max_characters = *max_char;
        }
        let icon_node = coordinator.binding_entity(&icon_ac);
        if let Ok(mut color) = colors.get_mut(icon_node) {
            *color = match state {
                ButtonStyle::Ring => foreground_color.0,
                ButtonStyle::Fill => background_color.0,
            };
        }
        if let Ok(mut scale) = scales.get_mut(icon_node) {
            *scale = icon_scale;
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
    pub fn new(
        style: ButtonStyle,
        text: TextValue,
        max_char: MaxCharacters,
        icon_id: IconId,
        foreground_color: Color,
        background_color: Color,
    ) -> Self {
        Self {
            style,
            text,
            max_char,
            icon_id,
            foreground_color,
            background_color,
        }
    }
}
fn button_metrics(
    area: Area<InterfaceContext>,
    max_char: MaxCharacters,
    font: &MonospacedFont,
    scale_factor: &ScaleFactor,
) -> (FontSize, f32, Area<InterfaceContext>, IconScale, i32) {
    let padding = 16.min((area.height / 4f32) as i32);
    let icon_scale = IconScale::from_dim(area.height - padding as f32);
    let text_area = area - (padding * 3, padding * 2).into() - (icon_scale.px(), 0.0).into();
    let (font_size, calculated_text_area) =
        font.best_fit(max_char, text_area.min_bound((0, 0)), scale_factor);
    let icon_left = area.width - icon_scale.px() - padding as f32 * 2f32;
    let diff = (icon_left - padding as f32 - calculated_text_area.width) / 2f32;
    let text_offset = padding as f32 + diff.max(0f32);
    (
        font_size,
        text_offset,
        calculated_text_area,
        icon_scale,
        padding,
    )
}
pub enum ButtonBindings {
    Panel,
    Text,
    Icon,
}
impl From<ButtonBindings> for SceneBinding {
    fn from(value: ButtonBindings) -> Self {
        SceneBinding::from(value as i32)
    }
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
        let (font_size, text_offset, _calc_area, icon_scale, padding) = button_metrics(
            anchor.0.section.area,
            args.max_char,
            &external_args.0,
            &external_args.1,
        );
        binder.bind(
            0,
            (0.near(), 0.near(), 1),
            Panel::new(
                PanelStyle::ring(),
                anchor.0.section.area,
                args.foreground_color,
            ),
            cmd,
        );
        binder.bind(
            1,
            (text_offset.near(), 0.center(), 0),
            Text::new(
                args.max_char,
                font_size,
                args.text.clone(),
                args.foreground_color,
            ),
            cmd,
        );
        binder.bind(
            2,
            (padding.far(), 0.center(), 0),
            Icon::new(args.icon_id, icon_scale, args.foreground_color),
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
