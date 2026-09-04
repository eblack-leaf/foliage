//! Polygon -- a regular n-sided shape, filled, with uniform corner rounding.

use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};

use crate::aspen::blend;
use crate::color::Color;
use crate::coordinate::Section;
use crate::elm::{Chlorophyll, Pigment};
use crate::op::Bud;
use crate::palette::Fill;
use crate::place::{Boxed, Caller, Placement, Places};
use crate::seed::Buds;

/// The fewest sides a shape can have.
const LEAST: f32 = 3.0;

/// A regular polygon: a triangle, a hexagon, a circle, and everything between.
///
/// The expressive shape beside [`Panel`](crate::Panel)'s rectangle, and deliberately not a
/// generalisation of it. A panel owns arbitrary-aspect boxes with independent per-corner radii; a
/// regular polygon's corners only stay circular while the shape stays square, so the two do not
/// collapse into one primitive and this is not trying to.
///
/// ```no_run
/// # use foliage::{Boxed, Location, Palette, Polygon, Source, left, top};
/// Polygon::new()
///     .sides(6.0)
///     .rounding(0.2)
///     .color(Palette::Accent)
///     .at(Location::new().xs(
///         left(0.px()).width(48.px()),
///         top(0.px()).height(48.px()),
///     ));
/// ```
///
/// The shape is inscribed in the largest circle its box holds, so a non-square box leaves room
/// around it rather than distorting it. That is what lets a composite size a polygon's box loosely
/// -- a fraction of its own geometry -- without having to reason about the aspect it lands at.
#[derive(Clone, Debug, Default)]
pub struct Polygon {
    pub(crate) placement: Placement,
    pub(crate) fill: Fill,
    pub(crate) shape: Shape,
}

impl Polygon {
    /// A surface-filled triangle with sharp corners, filling its trunk's box.
    pub fn new() -> Self {
        Self::default()
    }

    /// How many sides, clamped to at least three.
    ///
    /// Fractional counts are legal and are what makes the shape animatable: the field blends
    /// between the two whole counts either side, so a hexagon becomes a triangle by passing through
    /// the shapes between them.
    pub fn sides(mut self, sides: f32) -> Self {
        self.shape.sides = sides.max(LEAST);
        self
    }

    /// How round the corners are, from `0.0` sharp to `1.0` fully round.
    ///
    /// A fraction of the shape's own inradius rather than a pixel count, because a regular polygon's
    /// corner has no edge to measure a radius against the way a rectangle's does. At `1.0` it is a
    /// true circle whatever the side count, which is what makes a round dot and a hexagon the same
    /// element with a different number written on it.
    pub fn rounding(mut self, rounding: f32) -> Self {
        self.shape.rounding = rounding.clamp(0.0, 1.0);
        self
    }

    /// How far the shape is turned about its own centre, in radians.
    pub fn rotation(mut self, radians: f32) -> Self {
        self.shape.rotation = radians;
        self
    }

    /// What it is filled with: a [`Palette`](crate::Palette) role, or a [`Color`] stated outright.
    pub fn color(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();
        self
    }

    /// A circle: as many sides as it takes, fully rounded.
    ///
    /// The one shape worth a name of its own, because it is what a dot, a knob and the turn of a
    /// path all are, and `sides(3.0).rounding(1.0)` is a roundabout way of writing it.
    pub fn circle() -> Self {
        Self::new().rounding(1.0)
    }
}

impl Places for Polygon {
    fn placement(&mut self) -> &mut Placement {
        &mut self.placement
    }
}

impl Boxed for Polygon {}

impl Buds for Polygon {
    fn bud(self, at: Caller) -> Bud {
        Bud {
            chlorophyll: Chlorophyll::Polygon,
            pigment: Some(Pigment::Polygon(PolygonPigment {
                fill: self.fill,
                shape: self.shape,
            })),
            placement: self.placement,
            at,
            ..Bud::bare()
        }
    }
}

/// What a regular polygon looks like, as three numbers.
///
/// One value rather than three properties, because it is one thought and because
/// [`Motion::Polygon`](crate::Motion::Polygon) moves it as one: every field interpolates, and a
/// shape half way between two of these is a shape.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Shape {
    /// How many sides. At least three, and fractional counts blend between whole ones.
    pub sides: f32,
    /// How round the corners are, `0.0` sharp to `1.0` a true circle.
    pub rounding: f32,
    /// How far the shape is turned about its centre, in radians.
    pub rotation: f32,
}

impl Shape {
    /// A shape a fraction `at` of the way to `other`, field by field.
    ///
    /// Plainly channel-wise, and that is the whole of it: sides, rounding and rotation are numbers,
    /// so a polygon needs none of the machinery a placement's endpoints do.
    pub(crate) fn blend(self, other: Self, at: f32) -> Self {
        Self {
            sides: blend(self.sides, other.sides, at).max(LEAST),
            rounding: blend(self.rounding, other.rounding, at).clamp(0.0, 1.0),
            rotation: blend(self.rotation, other.rotation, at),
        }
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            sides: LEAST,
            rounding: 0.0,
            rotation: 0.0,
        }
    }
}

/// What the polygon renderer was told.
///
/// Grown alongside [`Chlorophyll::Polygon`] and by nothing else, so an element carries both or
/// neither.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct PolygonPigment {
    pub(crate) fill: Fill,
    pub(crate) shape: Shape,
}

/// One polygon, in the form the backend takes it.
///
/// `#[repr(C)]` over the types the coordinate module guarantees the layout of, so what extraction
/// compares and what the vertex buffer holds are the same bytes.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct PolygonInstance {
    pub(crate) section: Section,
    pub(crate) color: Color,
    /// Sides, rounding and rotation, in that order. Three floats rather than a [`Shape`] so the
    /// instance stays plainly the bytes a vertex buffer takes.
    pub(crate) shape: [f32; 3],
}

impl PolygonInstance {
    pub(crate) fn new(section: Section, color: Color, shape: Shape) -> Self {
        Self {
            section,
            color,
            shape: [shape.sides, shape.rounding, shape.rotation],
        }
    }
}
