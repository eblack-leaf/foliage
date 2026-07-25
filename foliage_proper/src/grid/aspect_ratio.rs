use crate::EcsExtension;
use crate::{CoordinateContext, Layout, Location, Position, Section, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::HookContext;
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
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
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
            // recenter the shrunk box within the original section on both axes -- leaving
            // `section.position` untouched pins the result to the original top-left corner,
            // silently shifting anything anchored to the shape's own center/edges away from
            // where every non-anchored sibling (a fixed row, a percent-based line) assumes
            // that center to be.
            let diff = Position::from((
                (section.width() - attempted_width) * 0.5,
                (section.height() - attempted_height) * 0.5,
            ));
            let constrained =
                Section::new(section.position + diff, (attempted_width, attempted_height));
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Logical;

    #[test]
    fn constraining_a_tall_box_to_square_centers_it_vertically_not_pinned_to_the_top() {
        // 50 wide x 88 tall -> square constrained to 50x50; the removed 38px of height
        // should come off evenly top and bottom, not all from the bottom.
        let section = Section::<Logical>::new((10.0, 20.0), (50.0, 88.0));
        let constrained = AspectRatio::new()
            .xs(1.0)
            .constrain(section, Layout::Xs)
            .unwrap();
        assert_eq!(constrained.width(), 50.0);
        assert_eq!(constrained.height(), 50.0);
        assert_eq!(
            constrained.left(),
            10.0,
            "left should be unchanged (width already matched)"
        );
        assert_eq!(
            constrained.top(),
            20.0 + 19.0,
            "top should shift down by half the removed height"
        );
        assert_eq!(constrained.center().left(), section.center().left());
        assert_eq!(constrained.center().top(), section.center().top());
    }

    #[test]
    fn constraining_a_wide_box_to_square_centers_it_horizontally_not_pinned_to_the_left() {
        let section = Section::<Logical>::new((10.0, 20.0), (88.0, 50.0));
        let constrained = AspectRatio::new()
            .xs(1.0)
            .constrain(section, Layout::Xs)
            .unwrap();
        assert_eq!(constrained.width(), 50.0);
        assert_eq!(constrained.height(), 50.0);
        assert_eq!(
            constrained.top(),
            20.0,
            "top should be unchanged (height already matched)"
        );
        assert_eq!(
            constrained.left(),
            10.0 + 19.0,
            "left should shift right by half the removed width"
        );
        assert_eq!(constrained.center().left(), section.center().left());
        assert_eq!(constrained.center().top(), section.center().top());
    }
}
