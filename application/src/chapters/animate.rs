use foliage::{
    Animation, Color, Ease, EcsExtension, Elevation, Entity, GridExt, Location, Opacity, Polygon,
    Sprout, Tree,
};

const FADE_IN: u64 = 400;

/// Chapter 5: a component as a tweenable value -- interpolating from its current value
/// to a target over time. Placeholder shape only; the real infographic for this concept
/// lands later.
pub fn build(tree: &mut Tree, slot: Entity) {
    let seq = tree.sequence();
    let panel = tree.branch(
        slot,
        Polygon::new()
            .sides(7.0)
            .rounding(0.2)
            .color(Color::orange(400))
            .at(Location::new().xs(
                35.pct().as_left().with(30.pct().as_width()),
                20.pct().as_top().with(20.pct().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(Opacity::new(0.0)),
    );
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(panel)
            .during(seq)
            .start(0)
            .finish(FADE_IN)
            .eased(Ease::DECELERATE),
    );
}
