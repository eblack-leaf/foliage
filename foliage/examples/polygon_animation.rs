//! Morphing and spinning `Polygon`s. Side count, corner rounding and rotation are plain
//! numbers, so a tween over three channels is the whole animation -- foliage supplies the
//! clock and the easing, the app decides the numbers are a shape. Three polygons run at
//! staggered periods so they drift out of phase. Run with
//! `cargo run --example polygon_animation -p foliage`.

use foliage::{
    Bloom, Canopy, Color, Ease, Elevation, Foliage, GridExt, Leaf, Location, Polygon, Timing, Tween,
};
use foliage::{Grows, Sprout};
use std::f32::consts::PI;

/// One morphing polygon: which element, which tween is driving it, and which leg it is on.
struct Morph {
    leaf: Leaf,
    tween: Tween,
    step: u32,
    period: u64,
}

/// A sharp triangle on even steps, a fully-rounded octagon on odd ones. Rotation keeps
/// climbing rather than resetting, so the spin carries across legs instead of snapping back.
fn shape(step: u32) -> Polygon {
    let rounded = step % 2 == 1;
    Polygon {
        sides: if rounded { 8.0 } else { 3.0 },
        rounding: if rounded { 1.0 } else { 0.0 },
        rotation: step as f32 * PI,
    }
}

impl Morph {
    /// Starts the next leg, tweening from the shape it currently holds to the next one.
    fn leg(&mut self, canopy: &mut Canopy) {
        let from = shape(self.step);
        let to = shape(self.step + 1);
        self.tween = canopy.tween(
            [
                (from.sides, to.sides),
                (from.rounding, to.rounding),
                (from.rotation, to.rotation),
            ],
            Timing::over(self.period).eased(Ease::DECELERATE),
        );
        self.step += 1;
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((420, 200));

    let mut morphs: Vec<Morph> = Vec::new();
    foliage.photosynthesize(move |canopy: &mut Canopy| {
        if morphs.is_empty() {
            for (i, period) in [2400u64, 3000, 3600].into_iter().enumerate() {
                let left = 40 + i as i32 * 130;
                let leaf = canopy.leaf(
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
                let mut morph = Morph {
                    leaf,
                    // replaced by `leg` immediately; a tween has no meaningful empty value
                    tween: canopy.tween([(0.0, 0.0)], Timing::over(1)),
                    step: 0,
                    period,
                };
                morph.leg(canopy);
                morphs.push(morph);
            }
        }

        for bloom in canopy.take() {
            match bloom {
                Bloom::Tween { tween, values } => {
                    if let Some(morph) = morphs.iter().find(|m| m.tween == tween) {
                        let leaf = morph.leaf;
                        canopy.polygon(
                            leaf,
                            Polygon {
                                sides: values[0],
                                rounding: values[1],
                                rotation: values[2],
                            },
                        );
                    }
                }
                // One leg finished; start the next. Endless, with no completion callback and
                // nothing registered on the engine side -- just an emission and a decision.
                Bloom::TweenDone(tween) => {
                    if let Some(index) = morphs.iter().position(|m| m.tween == tween) {
                        let mut morph = morphs.swap_remove(index);
                        morph.leg(canopy);
                        morphs.push(morph);
                    }
                }
                _ => {}
            }
        }
    });
}
