use crate::AsTree;
use crate::{CoordinateContext, Layout, Location, Position, Resolve, Section};
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::DeferredWorld;

#[derive(Component, Copy, Clone)]
#[component(on_insert = Self::on_insert)]
/// Forces a resolved [`Section`] to a width-over-height ratio, per
/// breakpoint.
///
/// Applied after the entity's `Location` resolves, so it constrains whatever box the
/// layout produced rather than replacing it. The constrained box stays centered on the
/// original, so anchors pointing at its center or edges land where siblings expect.
///
/// Falls back down the breakpoints like every responsive type; unset means unconstrained.
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
    /// An unconstrained ratio. Set at least one breakpoint for it to do anything.
    pub fn new() -> Self {
        Self {
            xs: None,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    /// Width over height at every breakpoint -- `1.0` square, `16.0 / 9.0` widescreen.
    pub fn xs(mut self, xs: f32) -> Self {
        self.xs = Some(xs);
        self
    }
    /// Width over height from the `sm` breakpoint up -- `1.0` square, `16.0 / 9.0` widescreen.
    pub fn sm(mut self, sm: f32) -> Self {
        self.sm = Some(sm);
        self
    }
    /// Width over height from the `md` breakpoint up -- `1.0` square, `16.0 / 9.0` widescreen.
    pub fn md(mut self, md: f32) -> Self {
        self.md = Some(md);
        self
    }
    /// Width over height from the `lg` breakpoint up -- `1.0` square, `16.0 / 9.0` widescreen.
    pub fn lg(mut self, lg: f32) -> Self {
        self.lg = Some(lg);
        self
    }
    /// Width over height at the `xl` breakpoint -- `1.0` square, `16.0 / 9.0` widescreen.
    pub fn xl(mut self, xl: f32) -> Self {
        self.xl = Some(xl);
        self
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.tree().send_to(Resolve::<Location>::new(), this);
    }
    /// The largest box of this ratio that fits *inside* `section`, centered on it.
    /// `None` when no ratio applies at `layout`.
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
            // Recentered on the original box rather than pinned to its top-left, so the
            // shape's own center and edges stay where anchored siblings resolve against
            // them.
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
    /// The smallest box of this ratio that *covers* `section` -- the opposite of
    /// [`constrain`](Self::constrain), overflowing rather than fitting within.
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
    /// The ratio in force at `layout`, falling back down the breakpoints.
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
