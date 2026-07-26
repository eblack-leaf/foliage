use foliage::{
    Anchor, Color, EcsExtension, Elevation, Entity, Location, Opacity, Polygon, Sprout, Tree,
    anchor,
};

/// This app's one established shadow convention (`chrome.rs`'s `build_shadow`,
/// `navigator.rs`'s `shadow_box`) -- left and down, never right. A shadow offset any
/// other direction reads as a mistake, not a stylistic choice, so these aren't
/// caller-configurable knobs.
pub const SHADOW_OFFSET_PX: i32 = 6;
pub const SHADOW_Y_OFFSET_PX: i32 = 4;

/// Spawns a muted `Polygon` copy of `target`, anchored to it (so it tracks `target`'s own
/// live size without the caller re-stating it) and offset left+down by the constants
/// above -- the offset alone is enough to read as a shadow past the front shape's own
/// edge, no separate corner/rotation trick needed. Starts as an invisible triangle
/// (`sides: 3.0, rounding: 0.0, Opacity(0.0)`), same starting state [`morph_in`] expects
/// on its own target -- call `morph_in` on the returned entity the same way you'd call it
/// on `target` itself (same `seq`, same stage list) to have the shadow morph in alongside
/// the front shape rather than just appearing.
///
/// Doesn't try to reactively mirror `target`'s Polygon during an ongoing animation --
/// `animate::<A>` writes the target component via a `Query` mutation each tick, not a
/// fresh `.insert()`, so `tree.react`/`tree.forward` (both `Trigger<Insert, C>`-based)
/// wouldn't see per-frame updates anyway. Running the shadow's own parallel animation
/// (via `morph_in`, same stage list, same `seq`) is the proven-correct way both existing
/// shadows in this app actually do it.
pub fn shadow_of(tree: &mut Tree, target: Entity, elevation: Elevation, color: Color) -> Entity {
    tree.leaf(
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(color)
            .at(Location::new().xs(
                anchor()
                    .center_x()
                    .as_center_x()
                    .adjust(-SHADOW_OFFSET_PX)
                    .with(anchor().width().as_width()),
                anchor()
                    .center_y()
                    .as_center_y()
                    .adjust(SHADOW_Y_OFFSET_PX)
                    .with(anchor().height().as_height()),
            ))
            .elevate(elevation)
            .with((Anchor::new(target), Opacity::new(0.0))),
    )
}
