//! Turning a stroke into the form the GPU draws it from.

use bytemuck::{Pod, Zeroable};

use crate::color::Color;
use crate::line::{Cap, LineInstance};

/// One stroke, in the form the GPU takes it: the segment itself, and how thick and how capped.
///
/// The **segment** rather than four corners, because the vertex stage places the quad and the
/// fragment stage measures distance to the segment -- so corners worked out on the CPU would be
/// carried only to be turned back into the segment they came from. What is derived here is what the
/// density decides: an axis-aligned stroke put on whole device pixels, and how wide the edge's
/// coverage ramp is.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct LineQuad {
    /// The two ends: the near one, then the far one.
    pub(crate) segment: [f32; 4],
    pub(crate) color: Color,
    /// Half the stroke's weight, which is the radius its field is offset by.
    pub(crate) half: f32,
    /// Whether the ends are round, as a number the shader can branch on.
    pub(crate) round: f32,
    /// Half the coverage ramp -- see [`feather`].
    pub(crate) feather: f32,
    /// How it and everything it is drawn inside of are turned and scaled where it is drawn.
    pub(crate) pose: crate::pose::Affine,
}

impl LineQuad {
    /// What `instance` draws as, on a display of this density.
    pub(crate) fn new(instance: LineInstance, scale: f32) -> Self {
        let round = match instance.cap {
            Cap::Round => 1.0,
            Cap::Butt => 0.0,
        };
        // Snapped where it runs along an axis on the surface itself: a turn takes it off the axis
        // and a scale off the pixel grid, and either way there is no edge left to put on a whole
        // pixel -- the feather is what draws it cleanly then, as at any other angle.
        let along = instance.from.x == instance.to.x || instance.from.y == instance.to.y;
        let (from, to, half) = match along && instance.pose.upright() {
            true => snapped(instance, scale),
            false => (instance.from, instance.to, instance.weight / 2.0),
        };
        Self {
            segment: [from.x, from.y, to.x, to.y],
            color: instance.color,
            half,
            round,
            // One device pixel as drawn, which under a scale is a different length of the stroke's
            // own field.
            feather: feather(half, scale * instance.pose.magnification()),
            pose: instance.pose,
        }
    }
}

/// Half the stroke's coverage ramp, in the logical pixels its field is measured in.
///
/// The ramp spans exactly one device pixel, centred on the true edge. That width is the one at which
/// a linear ramp sampled at pixel centres sums to the coverage the shape actually has: wider, and
/// what a column of pixels sums to depends on where the centreline falls between them, which draws
/// as a stroke thinning and thickening along its own length as it drifts; narrower, and the edge
/// steps.
///
/// Stated here rather than derived in the shader because the density is the whole of the answer and
/// the density is known here. A screen-space derivative cannot state it. `fwidth` is `|dpdx| +
/// |dpdy|`, which over a distance field is `|cos| + |sin|` of the gradient's direction and so
/// reports between one and one and a half device pixels depending on the angle the stroke runs at;
/// it is taken per 2x2 quad, so whatever it reports is quantised across pairs of pixels; and it is
/// discontinuous along a butt cap, where the field's `max` changes which term is nearest.
///
/// Capped at half the weight, so a stroke thinner than its own ramp still reaches full coverage
/// along its centreline at any angle.
fn feather(half: f32, scale: f32) -> f32 {
    (0.5 / scale).min(half)
}

/// An axis-aligned stroke, with its edges on whole device pixels.
///
/// The same crisp alignment every other primitive gets from its own field, which a stroke has no
/// field to get. A rule whose centreline lands on a pixel boundary is split evenly across two rows,
/// so a one-pixel rule draws as two half-lit ones: the right amount of ink, twice the width, and
/// dimmer than it was asked to be.
///
/// The **thickness** is snapped and the centreline is then placed from it, rather than the other way
/// round: a whole number of device pixels of ink, sitting on the boundary, is the thing that reads
/// crisply. Snapping the centreline instead would put an even weight half a pixel out.
///
/// Axis-aligned only. At any other angle a stroke has no edge that *can* be snapped -- moving it
/// changes both its width and its angle -- and the shader's feather is what makes those read
/// cleanly instead.
fn snapped(
    instance: LineInstance,
    scale: f32,
) -> (
    crate::coordinate::Position,
    crate::coordinate::Position,
    f32,
) {
    let whole = |value: f32| (value * scale).round() / scale;
    // At least one device pixel of ink, whatever was asked for.
    let thickness = ((instance.weight * scale).round().max(1.0)) / scale;
    let half = thickness / 2.0;
    let (mut from, mut to) = (instance.from, instance.to);
    if from.y == to.y {
        let edge = whole(from.y - half);
        from.y = edge + half;
        to.y = edge + half;
        from.x = whole(from.x);
        to.x = whole(to.x);
    } else {
        let edge = whole(from.x - half);
        from.x = edge + half;
        to.x = edge + half;
        from.y = whole(from.y);
        to.y = whole(to.y);
    }
    (from, to, half)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Position;
    use crate::pose::Affine;

    fn stroke(from: (f32, f32), to: (f32, f32), pose: Affine) -> LineInstance {
        LineInstance {
            from: Position::new(from.0, from.1),
            to: Position::new(to.0, to.1),
            color: Color::rgb(1.0, 1.0, 1.0),
            weight: 4.0,
            cap: Cap::Butt,
            pose,
        }
    }

    /// The ramp is one device pixel as drawn: scaled up twice, a pixel is half as much of the
    /// stroke's own field, and scaled down, twice as much -- capped at half the weight as ever.
    #[test]
    fn a_scaled_stroke_feathers_one_pixel_as_drawn() {
        let at = |scale: f32| {
            let pose = Affine::about(Position::new(50.0, 50.0), 0.3, scale);
            LineQuad::new(stroke((0.0, 0.0), (100.0, 40.0), pose), 1.0).feather
        };
        assert_eq!(at(1.0), 0.5);
        assert!((at(2.0) - 0.25).abs() < 1e-5, "{}", at(2.0));
        assert!((at(0.5) - 1.0).abs() < 1e-5, "{}", at(0.5));
    }

    /// Along an axis and upright, a stroke is put on whole pixels. Turned or scaled, it has no
    /// pixel edge left to be put on, and is drawn where it was asked to be.
    #[test]
    fn only_an_upright_stroke_is_snapped() {
        let upright = LineQuad::new(stroke((0.0, 10.3), (100.0, 10.3), Affine::IDENTITY), 1.0);
        assert_ne!(upright.segment[1], 10.3);
        let scaled = Affine::about(Position::new(50.0, 50.0), 0.0, 1.5);
        let posed = LineQuad::new(stroke((0.0, 10.3), (100.0, 10.3), scaled), 1.0);
        assert_eq!(posed.segment[1], 10.3);
        assert_eq!(posed.half, 2.0);
    }
}
