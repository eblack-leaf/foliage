use crate::anim::interpolation::Interpolations;
use crate::ash::clip::ClipContext;
use crate::grid::AspectRatio;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Animate, Attachment, Color, Component, Differential, Foliage, LeafSprout, Logical,
    ResolvedElevation, Section, Sprout, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};

mod pipeline;

/// A regular N-sided polygon, filled, with uniform corner rounding -- the "expressive
/// shape" primitive: `sides`/`rounding`/`rotation` are plain animatable scalars, so
/// morphing circle<->hexagon<->triangle or sharp<->round is just interpolating them, the
/// same mechanism already driving `Color`/`Location`. Side-count changes are a distance-
/// field blend in the shader (`polygon.wgsl`), not a vertex-matched morph -- cheap, and
/// every rounded endpoint already lacks the acute unrounded corner that would make a
/// blend look wrong mid-transition.
///
/// Deliberately separate from [`crate::Panel`]: `Panel` owns arbitrary-aspect rectangles
/// with independent per-corner radii and borders and is used everywhere already; a
/// regular polygon's rounded corners only stay circular if the shape stays roughly
/// square, so this doesn't generalize to `Panel`'s job and isn't trying to. Placed like
/// `Panel`/`Icon` (a bounding box via `.at(Location::new().xs(..))`), not like `Line`'s
/// point-mode.
#[repr(C)]
#[derive(Component, Pod, Zeroable, Copy, Clone, Debug, PartialEq)]
#[require(Differential<Self, Section<Logical>>)]
#[require(Color, Differential<Self, Color>)]
#[require(Differential<Self, ResolvedElevation>)]
#[require(Differential<Self, BlendedOpacity>)]
#[require(Differential<Self, ClipContext>)]
#[require(Differential<Self, Self>)]
#[component(on_add = Self::on_add)]
pub struct Polygon {
    pub sides: f32,
    pub rounding: f32,
    pub rotation: f32,
}
impl Default for Polygon {
    fn default() -> Self {
        Self {
            sides: 3.0,
            rounding: 0.0,
            rotation: 0.0,
        }
    }
}
impl Polygon {
    pub fn new() -> PolygonSprout {
        PolygonSprout::default()
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Remove::push_remove_packet::<Self>)
            .observe(Visibility::push_remove_packet::<Self>);
    }
}
impl Animate for Polygon {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new()
            .with(start.sides, end.sides)
            .with(start.rounding, end.rounding)
            .with(start.rotation, end.rotation)
    }
    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(sides) = interpolations.read(0) {
            self.sides = sides;
        }
        if let Some(rounding) = interpolations.read(1) {
            self.rounding = rounding;
        }
        if let Some(rotation) = interpolations.read(2) {
            // Normalized here, not left to grow -- callers routinely animate "current +
            // one more turn" (a click-spin, a re-triggered continuous spin) rather than
            // a fixed absolute target, since a fixed target only visibly moves once, so
            // `rotation` only ever grows across a session with that pattern. Safe to
            // normalize post-hoc: `interpolations` captured its own start/end once at
            // animation setup and computes each frame's raw value from that, independent
            // of whatever this write does to `self.rotation` afterward, so this can't
            // introduce a mid-tween jump. The *next* animation reads its own `start` from
            // this (now-normalized) live value, so "current + delta" still produces the
            // exact same visible motion, just off a small number instead of an
            // ever-growing one. `rem_euclid`, not `%` -- `%` on a negative value (a
            // counter-clockwise delta) returns a negative remainder in Rust, not a
            // canonical angle.
            self.rotation = rotation.rem_euclid(2.0 * std::f32::consts::PI);
        }
    }
}
impl Attachment for Polygon {
    fn attach(foliage: &mut Foliage) {
        foliage.remove_queue::<Self>();
        foliage.differential::<Self, Section<Logical>>();
        foliage.differential::<Self, Self>();
        foliage.differential::<Self, Color>();
        foliage.differential::<Self, ResolvedElevation>();
        foliage.differential::<Self, ClipContext>();
        foliage.differential::<Self, BlendedOpacity>();
        foliage.enable_animation::<Self>();
    }
}
pub struct PolygonSprout {
    leaf: LeafSprout,
    color: Option<Color>,
    polygon: Polygon,
}
impl Default for PolygonSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            color: None,
            polygon: Polygon::default(),
        }
    }
}
impl Sprout for PolygonSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        // A regular polygon's rounded corners only stay circular -- and the shader's
        // `min(width, height)` apothem clamp only agrees with a box's own declared edges
        // -- if the box is square. Without this, independent width%/height% resolution
        // (each against its own screen axis) lets a non-square box silently shrink the
        // visible shape inside its own bounds on any non-matching aspect ratio, same as
        // `Icon` constrains itself for the same reason.
        (
            self.polygon,
            self.color.unwrap_or_default(),
            AspectRatio::new().xs(1.0),
        )
    }
}
impl PolygonSprout {
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// Number of sides, clamped to >=3 -- fractional values are valid (they drive the
    /// shader's side-count blend) but a shape needs at least a triangle's worth.
    pub fn sides(mut self, n: f32) -> Self {
        self.polygon.sides = n.max(3.0);
        self
    }
    /// 0.0 (sharp) to 1.0 (fully round -- a true circle regardless of side count),
    /// normalized to the shape's own apothem the same way `Panel`'s `Rounding` steps are
    /// normalized to its box, so the vocabulary matches even though the type doesn't.
    pub fn rounding(mut self, r: f32) -> Self {
        self.polygon.rounding = r.clamp(0.0, 1.0);
        self
    }
    pub fn rotation(mut self, radians: f32) -> Self {
        self.polygon.rotation = radians;
        self
    }
}
