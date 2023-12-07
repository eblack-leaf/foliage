use crate::color::Color;
use crate::elm::{Elm, Leaf};
use crate::icon::{Icon, IconId, IconScale};
use crate::panel::{Panel, PanelStyle};
use crate::scene::align::{SceneAligner, SceneAnchor};
use crate::scene::bind::SceneBinder;
use crate::scene::Scene;
use crate::text::font::MonospacedFont;
use crate::text::{MaxCharacters, Text, TextValue};
use crate::window::ScaleFactor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Commands;

#[derive(Bundle)]
pub struct Button {
    // custom data here
}
impl Leaf for Button {
    fn attach(elm: &mut Elm) {}
}
impl Scene for Button {
    type Args<'a> = (
        TextValue,
        MaxCharacters,
        IconId,
        Color,
        &'a MonospacedFont,
        &'a ScaleFactor,
    );

    fn bind_nodes<'a>(
        cmd: &mut Commands,
        anchor: SceneAnchor,
        args: &Self::Args<'a>,
        binder: &mut SceneBinder,
    ) -> Self {
        let padding = 16.min((anchor.0.section.height() / 4f32) as i32);
        let icon_scale = IconScale::from_dim(anchor.0.section.height() - padding as f32);
        let text_area = anchor.0.section.area
            - (padding * 3, padding * 2).into()
            - (icon_scale.px(), 0.0).into();
        let (font_size, area) = args.4.best_fit(args.1, text_area.min_bound((0, 0)), args.5);
        let icon_left = anchor.0.section.area.width - icon_scale.px() - padding as f32 * 2f32;
        let diff = (icon_left - padding as f32 - area.width) / 2f32;
        binder.bind(
            0,
            (0.near(), 0.near(), 1),
            Panel::new(PanelStyle::ring(), anchor.0.section.area, args.3),
            cmd,
        );
        binder.bind(
            1,
            ((padding as f32 + diff.max(0f32)).near(), 0.center(), 0),
            Text::new(args.1, font_size, args.0.clone(), args.3),
            cmd,
        );
        binder.bind(
            2,
            (padding.far(), 0.center(), 0),
            Icon::new(args.2, icon_scale, args.3),
            cmd,
        );
        Self {}
    }
}