//! Animating `Polygon`. Its `sides`, `rounding`, and `rotation` are plain animatable scalars,
//! so morphing triangle <-> circle and spinning is just interpolating them -- the same
//! `Animation`/`Sequence` mechanism that drives `Color`/`Location`, nothing polygon-specific.
//! Three polygons morph and spin endlessly at staggered periods so they drift out of phase.
//! Run with `cargo run --example polygon_animation -p foliage`.

use foliage::{
    Animation, Color, Ease, EcsExtension, Elevation, Entity, Foliage, GridExt, Location, OnEnd,
    Polygon, Sequence, Sprout, Tree, Trigger,
};
use std::f32::consts::PI;

/// One leg of an endless morph+spin: interpolate `target` toward the next shape over `period`
/// ms, then on completion schedule the following leg. `step` alternates the shape between a
/// sharp triangle and a fully-rounded octagon, and feeds the rotation (`step * PI`, ever
/// increasing) so the spin carries continuously across legs instead of snapping back.
fn drive<T: EcsExtension>(tree: &mut T, target: Entity, step: u32, period: u64) {
    let rounded = step % 2 == 1;
    let shape = Polygon {
        sides: if rounded { 8.0 } else { 3.0 },
        rounding: if rounded { 1.0 } else { 0.0 },
        rotation: step as f32 * PI,
    };
    Sequence::new(tree)
        .animate(
            Animation::new(shape)
                .targeting(target)
                .start(0)
                .finish(period)
                .eased(Ease::DECELERATE),
        )
        .end(move |_: Trigger<OnEnd>, mut tree: Tree| {
            drive(&mut tree, target, step + 1, period);
        });
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((420, 200));

    let periods = [2400u64, 3000, 3600];
    for (i, period) in periods.into_iter().enumerate() {
        let left = 40 + i as i32 * 130;
        let polygon = foliage.world.leaf(
            Polygon::new()
                .sides(3.0)
                .rounding(0.0)
                .color(Color::green(300))
                .at(Location::new().xs(
                    left.px().as_left().with(100.px().as_width()),
                    50.px().as_top().with(100.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        // start at step 1: the leaves already sit at the step-0 shape (sharp triangle), so the
        // first leg morphs them *to* the octagon rather than animating triangle -> triangle.
        drive(&mut foliage.world, polygon, 1, period);
    }

    foliage.photosynthesize();
}
