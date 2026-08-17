//! Corner rounding, shared by every renderer that draws a rounded rectangle rather than
//! owned by any one of them -- [`Panel`](crate::Panel) and [`Image`](crate::Image) resolve
//! the same brackets through the same [`CornerRadii`] and hand their shaders the same four
//! numbers, which is what lets a full-bleed image sit flush inside a rounded panel.

use crate::{
    AsTree, Component, CoordinateUnit, InteractionShape, Panel, Physical, Resolve, Section,
};
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;

/// Corner radius as a bracket rather than a raw pixel count, so radii stay consistent
/// across a UI and scale with the element.
///
/// Resolved against the element's own shorter side, so `Md` on a small chip and on a large
/// card read as the same treatment. [`Full`](Rounding::Full) rounds to a pill or circle
/// and switches the hit test to match. Which corners are affected is [`Side`]'s job.
#[derive(Component, Copy, Clone, Default, Eq, PartialEq)]
#[component(on_insert = Self::on_insert)]
pub enum Rounding {
    #[default]
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Full,
}
impl Rounding {
    /// The radius this bracket resolves to for `section`, in `section`'s own units.
    ///
    /// `Xs`-`Lg` are fixed logical-pixel radii -- the same curve on a small chip and a large
    /// card, which is what "the same treatment" actually looks like; a fraction of the box
    /// grows into a distorted, oversized curve as the box does. [`Full`](Rounding::Full) is
    /// the one bracket that has to scale with the box instead: a pill/circle's radius is
    /// `height / 2` by definition, not a fixed number, so it stays a fraction of the shorter
    /// *half*-side, landing exactly on the largest radius the corner discs in `sdf.wgsl` stay
    /// exact at. Every fixed radius is clamped to that same ceiling, so a chip smaller than
    /// its own bracket can't ask for more curve than its box has room for.
    pub(crate) fn depth(self, section: Section<Physical>, scale_factor: CoordinateUnit) -> CoordinateUnit {
        let min = section.width().min(section.height()) * 0.5;
        match self {
            Rounding::None => 0.0,
            Rounding::Xs => (4.0 * scale_factor).min(min),
            Rounding::Sm => (8.0 * scale_factor).min(min),
            Rounding::Md => (12.0 * scale_factor).min(min),
            Rounding::Lg => (16.0 * scale_factor).min(min),
            Rounding::Full => min,
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        if *world.get::<Rounding>(this).unwrap() == Rounding::Full {
            world.tree().write_to(this, InteractionShape::Circle);
        } else {
            world.tree().write_to(this, InteractionShape::Rectangle);
        }
        // Only `Panel` needs poking: it resolves its radii from an observer, so a rounding
        // change with no section change would otherwise never reach it. `Image` resolves
        // its own in a `Changed<Rounding>` system, which this insert already satisfies.
        if world.get::<Panel>(this).is_some() {
            world.tree().send_to(Resolve::<Panel>::new(), this);
        }
    }
}
/// Which corners [`Rounding`] actually applies to -- composes independently of the amount
/// (a segmented control's middle segment wants `Side::none()`, its first wants
/// `Side::left()`, sharing one `Rounding` throughout). Defaults to all four.
#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Self::on_insert)]
pub struct Side {
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_left: bool,
    pub bottom_right: bool,
}
impl Default for Side {
    fn default() -> Self {
        Self::all()
    }
}
impl Side {
    /// Round every corner. The default.
    pub fn all() -> Self {
        Self {
            top_left: true,
            top_right: true,
            bottom_left: true,
            bottom_right: true,
        }
    }
    /// Square every corner, leaving [`Rounding`] with nothing to apply to.
    pub fn none() -> Self {
        Self {
            top_left: false,
            top_right: false,
            bottom_left: false,
            bottom_right: false,
        }
    }
    /// Round the two left corners only -- the leading end of a segmented row.
    pub fn left() -> Self {
        Self {
            top_left: true,
            bottom_left: true,
            top_right: false,
            bottom_right: false,
        }
    }
    /// Round the two right corners only -- the trailing end of a segmented row.
    pub fn right() -> Self {
        Self {
            top_left: false,
            bottom_left: false,
            top_right: true,
            bottom_right: true,
        }
    }
    /// Round the two top corners only.
    pub fn top() -> Self {
        Self {
            top_left: true,
            top_right: true,
            bottom_left: false,
            bottom_right: false,
        }
    }
    /// Round the two bottom corners only.
    pub fn bottom() -> Self {
        Self {
            top_left: false,
            top_right: false,
            bottom_left: true,
            bottom_right: true,
        }
    }
    /// Name each corner individually, for combinations the presets do not cover.
    pub fn corners(top_left: bool, top_right: bool, bottom_left: bool, bottom_right: bool) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        if world.get::<Panel>(this).is_some() {
            world.tree().send_to(Resolve::<Panel>::new(), this);
        }
    }
}
/// The four radii `sdf.wgsl` reads, in physical pixels and in its field order.
///
/// Resolved from [`Rounding`] and [`Side`] against the element's own box rather than
/// authored. Renderers embed it in whatever they already ship -- `Panel` carries it inside
/// the `Panel` packet itself, `Image` as its own attribute -- so sharing the shape costs
/// neither of them a change to how it differentials.
#[repr(C)]
#[derive(Component, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub(crate) struct CornerRadii {
    pub(crate) top_left: CoordinateUnit,
    pub(crate) top_right: CoordinateUnit,
    pub(crate) bottom_left: CoordinateUnit,
    pub(crate) bottom_right: CoordinateUnit,
}
impl CornerRadii {
    pub(crate) fn resolve(
        section: Section<Physical>,
        rounding: Rounding,
        side: Side,
        scale_factor: CoordinateUnit,
    ) -> Self {
        let depth = rounding.depth(section, scale_factor);
        let of = |rounded: bool| if rounded { depth } else { 0.0 };
        Self {
            top_left: of(side.top_left),
            top_right: of(side.top_right),
            bottom_left: of(side.bottom_left),
            bottom_right: of(side.bottom_right),
        }
    }
}
