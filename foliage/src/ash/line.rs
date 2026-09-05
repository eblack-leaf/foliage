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
}

impl LineQuad {
    /// What `instance` draws as, on a display of this density.
    pub(crate) fn new(instance: LineInstance, scale: f32) -> Self {
        let round = match instance.cap {
            Cap::Round => 1.0,
            Cap::Butt => 0.0,
        };
        let (from, to, half) =
            match instance.from.x == instance.to.x || instance.from.y == instance.to.y {
                true => snapped(instance, scale),
                false => (instance.from, instance.to, instance.weight / 2.0),
            };
        Self {
            segment: [from.x, from.y, to.x, to.y],
            color: instance.color,
            half,
            round,
            feather: feather(half, scale),
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
