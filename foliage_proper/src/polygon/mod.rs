use crate::AsTree;
use crate::anim::interpolation::Interpolations;
use crate::ash::clip::ClipContext;
use crate::grid::AspectRatio;
use crate::opacity::BlendedOpacity;
use crate::remove::Remove;
use crate::{
    Animate, Attachment, Author, Color, Component, Differential, Foliage, LeafSprout, Logical,
    ResolvedElevation, Section, Visibility,
};
use bevy_ecs::bundle::Bundle;
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
    /// Starts a [`Polygon`] entity. Set [`sides`](PolygonSprout::sides) and the rest on
    /// the builder; placed by a bounding box like [`Panel`](crate::Panel), not in point
    /// mode like [`Line`](crate::Line).
    pub fn new() -> PolygonSprout {
        PolygonSprout::default()
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let mut tree = world.tree();
        tree.subscribe(this, Remove::push_remove_packet::<Self>);
        tree.subscribe(this, Visibility::push_remove_packet::<Self>);
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
            self.rotation = rotation;
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
/// Builder for a [`Polygon`] entity -- see [`Polygon::new`].
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
impl Author for PolygonSprout {
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
    /// Fill color.
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
    /// Rotation about the shape's own center, in radians.
    pub fn rotation(mut self, radians: f32) -> Self {
        self.polygon.rotation = radians;
        self
    }
}
