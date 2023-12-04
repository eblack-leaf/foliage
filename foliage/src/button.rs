use crate::color::Color;
use crate::icon::IconId;
use crate::panel::{Panel, PanelStyle};
use crate::scene::{HorizontalAlignment, LayerAlignment, Scene, SceneNode, VerticalAlignment};
use crate::text::FontSize;
use bevy_ecs::entity::Entity;
use std::marker::PhantomData;
#[derive(Clone)]
pub struct Button<T: Clone> {
    font_size: FontSize,
    icon_id: IconId,
    background_color: Color,
    _phantom: PhantomData<T>,
}
impl<T: Clone> Button<T> {
    pub fn new(font_size: FontSize, icon_id: IconId, background_color: Color) -> Self {
        Self {
            font_size,
            icon_id,
            background_color,
            _phantom: PhantomData,
        }
    }
}
impl<T: Clone + Send> Scene for Button<T> {
    fn nodes(&self) -> Vec<SceneNode<Self>> {
        vec![SceneNode::new(
            0,
            HorizontalAlignment::Left(0f32),
            VerticalAlignment::Top(0f32),
            LayerAlignment::Layer(1f32),
            |b, extent, cmd, _scale_factor| -> Entity {
                cmd.spawn(Panel::new(PanelStyle::ring(), extent, b.background_color))
                    .id()
            },
        )]
    }
}
