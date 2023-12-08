use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::elm::{Elm, Leaf, SystemSets, Tag};
use crate::icon::{Icon, IconId, IconScale};
use crate::panel::{Panel, PanelStyle};
use crate::scene::align::{PositionAlignment, SceneAligner, SceneAnchor};
use crate::scene::bind::{SceneBinder, SceneNodes};
use crate::scene::Scene;
use crate::text::font::MonospacedFont;
use crate::text::{FontSize, MaxCharacters, Text, TextValue};
use crate::window::ScaleFactor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::prelude::{Changed, Commands, Component, IntoSystemConfigs};
use bevy_ecs::query::{Or, With, Without};
use bevy_ecs::system::Query;

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
impl Leaf for Button {
    fn attach(elm: &mut Elm) {
        elm.job
            .main()
            .add_systems((updates.in_set(SystemSets::Prepare),));
    }
}
#[derive(Copy, Clone, Component)]
pub struct ForegroundColor(pub Color);
#[derive(Copy, Clone, Component)]
pub struct BackgroundColor(pub Color);
fn updates(
    query: Query<
        (
            &Area<InterfaceContext>,
            &MaxCharacters,
            &SceneNodes,
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
    mut alignments: Query<&mut PositionAlignment>,
    mut scales: Query<&mut IconScale>,
    mut font_sizes: Query<(&mut FontSize, &mut MaxCharacters), Without<Tag<Button>>>,
    mut colors: Query<&mut Color>,
    mut panel_styles: Query<&mut PanelStyle>,
) {
    for (button_area, max_char, nodes, foreground_color, background_color, state) in query.iter() {
        let (fs, text_offset, text_area, icon_scale, padding) =
            button_metrics(*button_area, *max_char, &font, &scale_factor);
        let panel_node = nodes.get(0).entity();
        if let Ok(mut color) = colors.get_mut(panel_node) {
            *color = match state {
                ButtonStyle::Ring => foreground_color.0,
                ButtonStyle::Fill => foreground_color.0,
            };
        }
        if let Ok(mut style) = panel_styles.get_mut(panel_node) {
            *style = match state {
                ButtonStyle::Ring => {
                    PanelStyle::ring()
                }
                ButtonStyle::Fill => {
                    PanelStyle::fill()
                }
            }
        }
        let text_node = nodes.get(1).entity();
        if let Ok(mut alignment) = alignments.get_mut(text_node) {
            alignment.horizontal = text_offset.near();
        }
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
        let icon_node = nodes.get(2).entity();
        if let Ok(mut color) = colors.get_mut(icon_node) {
            *color = match state {
                ButtonStyle::Ring => foreground_color.0,
                ButtonStyle::Fill => background_color.0,
            };
        }
        if let Ok(mut alignment) = alignments.get_mut(icon_node) {
            alignment.horizontal = padding.far();
        }
        if let Ok(mut scale) = scales.get_mut(icon_node) {
            *scale = icon_scale;
        }
    }
}
pub struct ButtonArgs<'a> {
    style: ButtonStyle,
    text: TextValue,
    max_char: MaxCharacters,
    icon_id: IconId,
    foreground_color: Color,
    background_color: Color,
    font: &'a MonospacedFont,
    scale_factor: &'a ScaleFactor,
}
impl<'a> ButtonArgs<'a> {
    pub fn new(
        style: ButtonStyle,
        text: TextValue,
        max_char: MaxCharacters,
        icon_id: IconId,
        foreground_color: Color,
        background_color: Color,
        font: &'a MonospacedFont,
        scale_factor: &'a ScaleFactor,
    ) -> Self {
        Self {
            style,
            text,
            max_char,
            icon_id,
            foreground_color,
            background_color,
            font,
            scale_factor,
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
impl Scene for Button {
    type Args<'a> = ButtonArgs<'a>;

    fn bind_nodes<'a>(
        cmd: &mut Commands,
        anchor: SceneAnchor,
        args: &Self::Args<'a>,
        binder: &mut SceneBinder,
    ) -> Self {
        let (font_size, text_offset, _calc_area, icon_scale, padding) = button_metrics(
            anchor.0.section.area,
            args.max_char,
            args.font,
            args.scale_factor,
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
            tag: Tag::new(),
            foreground_color: ForegroundColor(args.foreground_color),
            background_color: BackgroundColor(args.background_color),
            max_characters: args.max_char,
            button_style: args.style,
            base_style: BaseStyle(args.style)
        }
    }
}