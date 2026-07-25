use foliage::{
    Animation, Color, Ease, EcsExtension, Elevation, Entity, GridExt, Location, Opacity, Polygon,
    Sprout, Tree,
};

const FADE_IN: u64 = 400;

/// Placeholder third route -- exists purely so back/forward navigation has more than one
/// hop to exercise (index 0 <-> 1 <-> 2, boundary polygons muted at each end). A static
/// pentagon, fading in, distinct in shape/color from `next`'s hexagon so the pages are easy
/// to tell apart on screen.
pub fn third(tree: &mut Tree, slot: Entity) {
    let seq = tree.sequence();
    let panel = tree.branch(
        slot,
        Polygon::new()
            .sides(5.0)
            .rounding(0.2)
            .color(Color::purple(400))
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
