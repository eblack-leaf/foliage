use crate::color::Color;
use crate::coordinate::{Coordinate, InterfaceContext};
use crate::elm::{Elm, Leaf};
use crate::icon::{Icon, IconId, IconScale};
use crate::job::Tag;
use crate::panel::{Panel, PanelStyle};
use crate::scene::{Scene, SceneBindRequest, SceneLayout};
use crate::text::font::MonospacedFont;
use crate::text::{MaxCharacters, Text, TextValue};
use crate::window::ScaleFactor;
use bevy_ecs::bundle::Bundle;
#[derive(Bundle)]
pub struct Button {
    tag: Tag<Button>,
    scene: Scene,
    panel_req: SceneBindRequest<Panel>,
    text_req: SceneBindRequest<Text>,
    icon_req: SceneBindRequest<Icon>,
    color: Color,
    icon_id: IconId,
    text_value: TextValue,
}
impl Button {
    pub fn new(
        coordinate: Coordinate<InterfaceContext>,
        icon_id: IconId,
        icon_scale: IconScale,
        text_value: TextValue,
        max_characters: MaxCharacters,
        color: Color,
        font: &MonospacedFont,
        scale_factor: &ScaleFactor,
    ) -> Self {
        let (font_size, _area) = font.best_fit(
            max_characters,
            coordinate.section.area.to_device(scale_factor.factor()),
        );
        Self {
            tag: Tag::new(),
            scene: Scene::new(coordinate, Self::layout()),
            panel_req: SceneBindRequest::new(vec![(
                0,
                0,
                Panel::new(PanelStyle::ring(), coordinate.section.area, color),
            )]),
            text_req: SceneBindRequest::new(vec![(
                1,
                1,
                Text::new(max_characters, font_size, text_value.clone(), color),
            )]),
            icon_req: SceneBindRequest::new(vec![(2, 2, Icon::new(icon_id, icon_scale, color))]),
            color,
            icon_id,
            text_value,
        }
    }
    fn layout() -> SceneLayout {
        todo!()
    }
}
impl Leaf for Button {
    fn attach(elm: &mut Elm) {
        elm.enable_scene_bind::<Panel>();
        elm.enable_scene_bind::<Text>();
        elm.enable_scene_bind::<Icon>();
    }
}

// forward color
//
