//! Pose -- how an element is turned, scaled and slid where it is drawn.
//!
//! All three are draw-time, as a [`shift`](crate::Grow::shift) is: where the layout put the element
//! does not change, so nothing placed against it moves, and nothing is laid out again for a pose.
//! What changes is where it is drawn and where it is pressed, and for everything grown under it as
//! well -- a card scaled down on a press scales its text with it, about the card's own middle.
//!
//! A turn and a scale are an [`Affine`] map of the plane, composed down the tree in R4 beside the
//! drawn box: an element's own about its own middle, inside whatever its trunk's is. Every renderer
//! applies it to the quad it lays out, after laying it out, so the distance fields that draw a round
//! corner, a polygon or a mark are still evaluated in the element's own space and their edges stay
//! one pixel wide whatever the map does to them. The box stack keeps its inverse, so a press is
//! tested in that same space.
//!
//! A slide is a shift measured in the element's own size rather than in pixels, which is what moves
//! a sheet out by exactly its own width without asking how wide it is.

use bytemuck::{Pod, Zeroable};

use crate::coordinate::{Position, Section};

/// How an element is turned, scaled and slid where it is drawn, as it declared it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Pose {
    /// Clockwise, in radians, about its middle.
    pub(crate) turn: f32,
    /// How many times its own size it is drawn, about its middle.
    pub(crate) scale: f32,
    /// How far it is drawn from where the layout put it, in its own widths and heights.
    pub(crate) slide: Position,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            turn: 0.0,
            scale: 1.0,
            slide: Position::default(),
        }
    }
}

impl Pose {
    /// Whether it turns or scales anything: the map it contributes is not the identity.
    pub(crate) fn upright(&self) -> bool {
        self.turn == 0.0 && self.scale == 1.0
    }
}

/// A map of the plane that keeps straight lines straight: a turn and a scale, and where it moves
/// the origin to. What an element and everything it is drawn inside of do to where it is drawn.
///
/// Laid out as the GPU reads it, beside every instance: the linear part's two columns, then the
/// offset. A point `p` goes to `(l0·x + l2·y + o0, l1·x + l3·y + o1)`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct Affine {
    pub(crate) linear: [f32; 4],
    pub(crate) offset: [f32; 2],
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine {
    /// Moves nothing.
    pub(crate) const IDENTITY: Self = Self {
        linear: [1.0, 0.0, 0.0, 1.0],
        offset: [0.0, 0.0],
    };

    /// Turns by `turn` clockwise and scales by `scale`, both about `middle`.
    pub(crate) fn about(middle: Position, turn: f32, scale: f32) -> Self {
        let (sin, cos) = turn.sin_cos();
        let (a, b) = (cos * scale, sin * scale);
        // Where the middle goes under the linear part alone, taken back to where it was.
        let moved = Position::new(a * middle.x - b * middle.y, b * middle.x + a * middle.y);
        Self {
            linear: [a, b, -b, a],
            offset: [middle.x - moved.x, middle.y - moved.y],
        }
    }

    /// This one, and then `outer`: what an element's own map is once its trunk's is applied over
    /// it.
    pub(crate) fn then(self, outer: Self) -> Self {
        let [a, b, c, d] = self.linear;
        let [e, f, g, h] = outer.linear;
        let moved = outer.apply(Position::new(self.offset[0], self.offset[1]));
        Self {
            linear: [e * a + g * b, f * a + h * b, e * c + g * d, f * c + h * d],
            offset: [moved.x, moved.y],
        }
    }

    /// Where it takes `at`.
    pub(crate) fn apply(&self, at: Position) -> Position {
        let [a, b, c, d] = self.linear;
        Position::new(
            a * at.x + c * at.y + self.offset[0],
            b * at.x + d * at.y + self.offset[1],
        )
    }

    /// The map that takes back what this one moved, or `None` where it flattened the plane -- a
    /// scale of zero, under which nothing is drawn and nothing can be pressed.
    pub(crate) fn inverse(&self) -> Option<Self> {
        let [a, b, c, d] = self.linear;
        let determinant = a * d - b * c;
        if determinant.abs() < f32::EPSILON {
            return None;
        }
        let linear = [
            d / determinant,
            -b / determinant,
            -c / determinant,
            a / determinant,
        ];
        let undo = Self {
            linear,
            offset: [0.0, 0.0],
        };
        let back = undo.apply(Position::new(self.offset[0], self.offset[1]));
        Some(Self {
            linear,
            offset: [-back.x, -back.y],
        })
    }

    /// The smallest upright box holding all of `section` once it is moved.
    pub(crate) fn bounds(&self, section: Section) -> Section {
        if *self == Self::IDENTITY {
            return section;
        }
        let corners = [
            Position::new(section.left(), section.top()),
            Position::new(section.right(), section.top()),
            Position::new(section.left(), section.bottom()),
            Position::new(section.right(), section.bottom()),
        ]
        .map(|corner| self.apply(corner));
        let (mut left, mut top) = (f32::INFINITY, f32::INFINITY);
        let (mut right, mut bottom) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for corner in corners {
            left = left.min(corner.x);
            top = top.min(corner.y);
            right = right.max(corner.x);
            bottom = bottom.max(corner.y);
        }
        Section::from_edges(left, top, right, bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: Position, b: Position) -> bool {
        (a.x - b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4
    }

    /// A turn is clockwise on a surface whose y runs down, and about the middle it names.
    #[test]
    fn a_quarter_turn_is_clockwise_about_its_middle() {
        let map = Affine::about(Position::new(10.0, 10.0), core::f32::consts::FRAC_PI_2, 1.0);
        assert!(near(
            map.apply(Position::new(10.0, 10.0)),
            Position::new(10.0, 10.0)
        ));
        // Right of the middle goes below it.
        assert!(near(
            map.apply(Position::new(20.0, 10.0)),
            Position::new(10.0, 20.0)
        ));
    }

    #[test]
    fn a_scale_is_about_its_middle() {
        let map = Affine::about(Position::new(10.0, 10.0), 0.0, 2.0);
        assert!(near(
            map.apply(Position::new(15.0, 10.0)),
            Position::new(20.0, 10.0)
        ));
    }

    /// Composing applies the inner map first, and the inverse takes back both.
    #[test]
    fn maps_compose_inside_out_and_undo() {
        let inner = Affine::about(Position::new(5.0, 5.0), 0.3, 1.5);
        let outer = Affine::about(Position::new(40.0, 0.0), -1.1, 0.5);
        let both = inner.then(outer);
        let at = Position::new(7.0, -3.0);
        assert!(near(both.apply(at), outer.apply(inner.apply(at))));
        let back = both.inverse().expect("invertible");
        assert!(near(back.apply(both.apply(at)), at));
    }

    #[test]
    fn a_scale_of_zero_cannot_be_undone() {
        assert!(
            Affine::about(Position::default(), 0.0, 0.0)
                .inverse()
                .is_none()
        );
    }

    #[test]
    fn bounds_hold_every_corner() {
        let map = Affine::about(Position::new(50.0, 50.0), core::f32::consts::FRAC_PI_4, 1.0);
        let bounds = map.bounds(Section::from_edges(40.0, 40.0, 60.0, 60.0));
        let half = 10.0 * core::f32::consts::SQRT_2;
        assert!((bounds.left() - (50.0 - half)).abs() < 1e-3);
        assert!((bounds.bottom() - (50.0 + half)).abs() < 1e-3);
    }
}
