use crate::color::Color;
use crate::elm::{Elm, Leaf};
use crate::icon::{Icon, IconId, IconScale};
use crate::panel::{Panel, PanelStyle};
use crate::r_scene::{Scene, SceneAligner, SceneAlignment, SceneAnchor, SceneBinder, SceneBinding};
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
        IconScale,
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
        let text_area = anchor.0.section.area - (24, 8).into() - (args.3.px(), args.3.px()).into();
        let (font_size, _area) = args.5.best_fit(args.1, text_area.min_bound((0, 0)), args.6);
        binder.bind(
            0,
            (0.near(), 0.near(), 1),
            Panel::new(PanelStyle::ring(), anchor.0.section.area, args.4),
            cmd,
        );
        binder.bind(
            1,
            (8.near(), 0.center(), 0),
            Text::new(args.1, font_size, args.0.clone(), args.4),
            cmd,
        );
        binder.bind(
            2,
            (8.far(), 0.center(), 0),
            Icon::new(args.2, args.3, args.4),
            cmd,
        );
        Self {}
    }
}
