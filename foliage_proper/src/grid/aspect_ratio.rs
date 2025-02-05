use crate::{CoordinateContext, Layout, Location, Position, Section, Update};
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
        layout: Layout,
    ) -> Option<Section<Context>> {
        if let Some(c) = self.config(layout) {
            let mut attempted_width = section.width();
            let mut attempted_height = attempted_width * 1.0 / c;
            while attempted_height > section.height() {
                attempted_width -= 1.0;
                attempted_height = attempted_width * 1.0 / c;
            }
            // let diff = Position::from((section.width() - attempted_width, 0.0)) * 0.5;
            let constrained = Section::new(section.position, (attempted_width, attempted_height));
            return Some(constrained);
        }
        None
    }
    pub fn fit<Context: CoordinateContext>(
        &self,
        section: Section<Context>,
        layout: Layout,
    ) -> Option<Section<Context>> {
        if let Some(c) = self.config(layout) {
            let mut attempted_width = section.width();
            let mut attempted_height = attempted_width * 1.0 / c;
            while attempted_height < section.height() {
                attempted_width += 1.0;
                attempted_height = attempted_width * 1.0 / c;
            }
            let diff = Position::from((section.width() - attempted_width, 0.0)) * 0.5;
            return Some(Section::new(
                section.position + diff,
                (attempted_width, attempted_height),
            ));
        }
        None
    }
    fn at_least_xs(&self) -> Option<f32> {
        if let Some(xs) = &self.xs {
            Some(*xs)
        } else {
            None
        }
    }
    fn at_least_sm(&self) -> Option<f32> {
        if let Some(sm) = &self.sm {
            Some(*sm)
        } else {
            self.at_least_xs()
        }
    }
    fn at_least_md(&self) -> Option<f32> {
        if let Some(md) = &self.md {
            Some(*md)
        } else {
            self.at_least_sm()
        }
    }
    fn at_least_lg(&self) -> Option<f32> {
        if let Some(lg) = &self.lg {
            Some(*lg)
        } else {
            self.at_least_md()
        }
    }
    fn at_least_xl(&self) -> Option<f32> {
        if let Some(xl) = &self.xl {
            Some(*xl)
        } else {
            self.at_least_lg()
        }
    }
    pub fn config(&self, layout: Layout) -> Option<f32> {
        match layout {
            Layout::Xs => self.at_least_xs(),
            Layout::Sm => self.at_least_sm(),
            Layout::Md => self.at_least_md(),
            Layout::Lg => self.at_least_lg(),
            Layout::Xl => self.at_least_xl(),
        }
    }
}
