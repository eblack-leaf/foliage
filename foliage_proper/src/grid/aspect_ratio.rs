use crate::{CoordinateContext, Location, Section, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::DeferredWorld;

#[derive(Component, Copy, Clone)]
#[component(on_insert = Self::on_insert)]
pub struct AspectRatio {
    pub xs: Option<f32>,
    pub sm: Option<f32>,
    pub md: Option<f32>,
    pub lg: Option<f32>,
    pub xl: Option<f32>,
}
impl Default for AspectRatio {
    fn default() -> Self {
        Self::new()
    }
}

impl AspectRatio {
    pub fn new() -> Self {
        Self {
            xs: None,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    pub fn xs(mut self, xs: f32) -> Self {
        self.xs = Some(xs);
        self
    }
    pub fn sm(mut self, sm: f32) -> Self {
        self.sm = Some(sm);
        self
    }
    pub fn md(mut self, md: f32) -> Self {
        self.md = Some(md);
        self
    }
    pub fn lg(mut self, lg: f32) -> Self {
        self.lg = Some(lg);
        self
    }
    pub fn xl(mut self, xl: f32) -> Self {
        self.xl = Some(xl);
        self
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Location>::new(), this);
    }
    pub fn constrain<Context: CoordinateContext>(
        &self,
        section: Section<Context>,
    ) -> Option<Section<Context>> {
        todo!()
    }
    pub fn fit<Context: CoordinateContext>(
        &self,
        section: Section<Context>,
    ) -> Option<Section<Context>> {
        todo!()
    }
}
