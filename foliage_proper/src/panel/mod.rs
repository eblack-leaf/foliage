use crate::AsTree;
use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::ash::clip::ClipContext;
use crate::ginkgo::ScaleFactor;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Animate, Animation, Attachment, Color, Component, CoordinateUnit, Coordinates, Differential,
    Foliage, InteractionShape, Logical, Position, Resolve, Resolved, ResolvedElevation, Section,
    Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;

mod pipeline;
mod vertex;

#[derive(Component, Copy, Clone, Default, PartialEq)]
#[require(Rounding, Side, Color, Outline)]
#[require(Differential<Self, ResolvedElevation>)]
#[require(Differential<Self, Color>)]
#[require(Differential<Self, Panel>)]
#[require(Differential<Self, Outline>)]
#[require(Differential<Self, Section<Logical>>)]
#[require(Differential<Self, BlendedOpacity>)]
#[require(Differential<Self, ClipContext>)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
/// A filled rectangle with optionally rounded corners and an optional outline -- the
/// background primitive nearly every composite is built on.
///
/// Spawned through [`Panel::new`]. Shape comes from three independent components:
/// [`Rounding`] (how much), [`Side`] (which corners), and [`Outline`] (ring instead of
/// fill). The per-corner geometry the shader consumes is computed from those, never
/// authored -- see `Panel::update`.
///
/// [`Rounding::Full`] also switches the entity's hit test to
/// [`InteractionShape::Circle`](crate::InteractionShape), so a pill or dot only responds
/// where it is actually drawn.
pub struct Panel {
    pub(crate) corner_i: Corner,
    pub(crate) corner_ii: Corner,
    pub(crate) corner_iii: Corner,
    pub(crate) corner_iv: Corner,
}
impl Panel {
    /// Starts a [`Panel`] entity:
    /// `canopy.branch(parent, Panel::new().color(c).rounding(Rounding::Md).at(loc))`.
    pub fn new() -> PanelSprout {
        PanelSprout::default()
    }
    pub(crate) fn new_marker() -> Panel {
        Panel {
            corner_i: Default::default(),
            corner_ii: Default::default(),
            corner_iii: Default::default(),
            corner_iv: Default::default(),
        }
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let mut tree = world.tree();
        tree.subscribe(this, Self::update_from_section);
        tree.subscribe(this, Remove::push_remove_packet::<Self>);
        tree.subscribe(this, Visibility::push_remove_packet::<Self>);
    }
    fn update_from_section(trigger: Trigger<Resolved<Section<Logical>>>, mut tree: Tree) {
        tree.send_to(Resolve::<Panel>::new(), trigger.event_target());
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.tree().send_to(Resolve::<Panel>::new(), this);
    }
    fn update(
        trigger: Trigger<Resolve<Panel>>,
        mut panels: Query<&mut Panel>,
        roundings: Query<&Rounding>,
        sides: Query<&Side>,
        outlines: Query<&Outline>,
        sections: Query<&Section<Logical>>,
        scale_factor: Res<ScaleFactor>,
    ) {
        let this = trigger.event_target();
        if panels.get(this).ok().is_none() {
            return;
        }
        if let Ok(section) = sections.get(this) {
            if let Ok(rounding) = roundings.get(this) {
                let side = sides.get(this).copied().unwrap_or_default();
                let section = section.to_physical(scale_factor.value()).rounded();
                let min = section.width().min(section.height()) * 0.5;
                let depth = match rounding {
                    Rounding::None => 0.0,
                    Rounding::Xs => 0.1 * min,
                    Rounding::Sm => 0.3 * min,
                    Rounding::Md => 0.5 * min,
                    Rounding::Lg => 0.7 * min,
                    Rounding::Full => 1.0 * min,
                };
                let weight = if let Ok(outline) = outlines.get(this) {
                    if outline.value.is_negative() {
                        None
                    } else {
                        Some(outline.value as f32 * scale_factor.value())
                    }
                } else {
                    None
                };
                if let Ok(mut panel) = panels.get_mut(this) {
                    let edge_adjust = 0.15;
                    // A squared corner is expressed as an unreachable radius, not a
                    // smaller mesh region: `panel.wgsl` insets every corner's quad by
                    // `corner_i`'s depth alone, so the corners cannot size independently.
                    // Pushing far/near past any distance inside the region leaves the
                    // curve unbent, which reads flush.
                    let sentinel = depth * 4.0 + 10.0;
                    let far = |rounded: bool| {
                        if rounded {
                            depth + edge_adjust
                        } else {
                            sentinel
                        }
                    };
                    let near = |rounded: bool| {
                        if !rounded {
                            return -edge_adjust;
                        }
                        if let Some(w) = weight {
                            // `+ edge_adjust`, matching `far`'s own sign rather than
                            // opposing it. Both radii shifting outward together moves the
                            // ring; shifting them apart widens it, and the ring's thickness
                            // is the outline's weight -- the same number the straight edges
                            // in `panel.wgsl` derive theirs from. Subtracting here made
                            // every corner `2 * edge_adjust` thicker than the edges it meets,
                            // which on a one-pixel outline is a third again as heavy.
                            depth - w.max(1.0) + edge_adjust
                        } else {
                            // A filled corner is the whole disc, so where its inner edge
                            // sits is arbitrary -- it only has to stay off the curve.
                            -edge_adjust
                        }
                    };
                    panel.corner_i = {
                        let c = Position::physical((depth, depth));
                        Corner::new(c.coordinates, far(side.top_left), near(side.top_left))
                    };
                    panel.corner_ii = {
                        let c = Position::physical((section.width() - depth, depth));
                        Corner::new(c.coordinates, far(side.top_right), near(side.top_right))
                    };
                    panel.corner_iii = {
                        let c = Position::physical((depth, section.height() - depth));
                        Corner::new(c.coordinates, far(side.bottom_left), near(side.bottom_left))
                    };
                    panel.corner_iv = {
                        let c =
                            Position::physical((section.width() - depth, section.height() - depth));
                        Corner::new(
                            c.coordinates,
                            far(side.bottom_right),
                            near(side.bottom_right),
                        )
                    };
                }
            }
        }
    }
}
impl Attachment for Panel {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Panel::update);
        foliage.define(Outline::update_anim);
        foliage.remove_queue::<Self>();
        foliage.differential::<Self, Section<Logical>>();
        foliage.differential::<Self, BlendedOpacity>();
        foliage.differential::<Self, Panel>();
        foliage.differential::<Self, Color>();
        foliage.differential::<Self, Outline>();
        foliage.differential::<Self, ResolvedElevation>();
        foliage.differential::<Self, ClipContext>();
        foliage.enable_animation::<Outline>();
    }
}
/// Builder for a [`Panel`] entity -- see [`Panel::new`].
#[derive(Default)]
pub struct PanelSprout {
    leaf: crate::LeafSprout,
    color: Option<Color>,
    rounding: Option<Rounding>,
    side: Option<Side>,
    outline: Option<i32>,
}
impl crate::Author for PanelSprout {
    fn seed(&mut self) -> &mut crate::LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Panel::new_marker(),
            self.color.unwrap_or_default(),
            self.rounding.unwrap_or_default(),
            self.side.unwrap_or_default(),
            self.outline.map(Outline::new).unwrap_or_default(),
        )
    }
}
impl PanelSprout {
    /// Fill color, or the ring's color when an [`Outline`] is set.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// Corner radius bracket. Applies to the corners [`Side`] names; defaults to square.
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.rounding = Some(r);
        self
    }
    /// Restricts [`Rounding`] to particular corners. Defaults to all four.
    pub fn side(mut self, s: Side) -> Self {
        self.side = Some(s);
        self
    }
    /// Draws a ring of this width in logical pixels instead of a solid fill. Squared
    /// corners fill solid rather than mitering -- see [`Outline`].
    pub fn outline(mut self, w: i32) -> Self {
        self.outline = Some(w);
        self
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
/// Corner radius as a bracket rather than a raw pixel count, so radii stay consistent
/// across a UI and scale with the panel.
///
/// Resolved against the panel's own shorter side, so `Md` on a small chip and on a large
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
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        if *world.get::<Rounding>(this).unwrap() == Rounding::Full {
            world.tree().write_to(this, InteractionShape::Circle);
        } else {
            world.tree().write_to(this, InteractionShape::Rectangle);
        }
        if world.get::<Panel>(this).is_some() {
            world.tree().send_to(Resolve::<Self>::new(), this);
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
pub(crate) struct Corner {
    pub(crate) coordinates: Coordinates,
    pub(crate) far: CoordinateUnit,
    pub(crate) near: CoordinateUnit,
}
impl Corner {
    pub(crate) fn new<C: Into<Coordinates>>(
        c: C,
        far: CoordinateUnit,
        near: CoordinateUnit,
    ) -> Self {
        Self {
            coordinates: c.into(),
            far,
            near,
        }
    }
}
/// Draws the panel as a ring of this width in logical pixels instead of a solid fill;
/// `0` is solid. Animatable, so a border can be drawn on.
///
/// The ring is computed from radial distance, which has no sharp-corner form: on a corner
/// [`Side`] leaves squared, the outline fills solid there rather than mitering a right
/// angle.
#[derive(Component, Copy, Clone, PartialEq)]
#[component(on_insert = Self::on_insert)]
pub struct Outline {
    pub value: i32,
}
impl Outline {
    /// A ring `value` logical pixels wide; `0` is a solid fill.
    pub fn new(value: i32) -> Outline {
        Outline { value }
    }
    fn update_anim(trigger: Trigger<Resolve<Animation<Self>>>, mut tree: Tree) {
        tree.send_to(Resolve::<Panel>::new(), trigger.event_target());
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        if world.get::<Panel>(this).is_some() {
            world.tree().send_to(Resolve::<Panel>::new(), this);
        }
    }
}
impl Default for Outline {
    fn default() -> Self {
        Outline { value: -1 }
    }
}
impl Animate for Outline {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.value as f32, end.value as f32)
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(o) = interpolations.read(0) {
            self.value = o as i32;
        }
    }
}
