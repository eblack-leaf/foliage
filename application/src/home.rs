use crate::navigator::NavigatorLanded;
use crate::type_in;
use foliage::{
    Color, EcsExtension, Elevation, Entity, FontSize, GridExt, Location, Rounding, Sprout,
    TextInput, Tree, Trigger,
};

/// The polygon/lines/icon "navigator" moved out to its own persistent module -- it
/// survives route switches, so it can no longer live inside any one scene. This route's
/// own content is just the type-in effect now. It doesn't start at this scene's own
/// t=0 (which, for the very first route, is immediately at boot, before the navigator's
/// intro has even begun) -- it waits for `NavigatorLanded`, targeted at this exact
/// `slot`. Home is revisitable, so this subscription is re-registered fresh on every
/// visit (a new `slot` each time) -- the navigator resends the event on every later
/// return, not just the first landing.
///
/// Deliberately NOT clamped to the same content-area bounds `toc.rs`'s viewport /
/// `chapters::window_frame` use -- tried that (a bounded container the same size), but on
/// a short landscape viewport that area is small enough that the type-in's own content
/// (headline + underline + subtitle + Docs button) has nowhere realistic to compress into
/// without reading as cramped; visibly bleeding past it on an extreme viewport is the
/// lesser problem.
pub fn home(tree: &mut Tree, slot: Entity) {
    tree.subscribe(slot, move |_: Trigger<NavigatorLanded>, mut tree: Tree| {
        let seq = tree.sequence();
        type_in::type_in(&mut tree, slot, seq, 0);
    });
    // TEMP: a focusable field for checking keyboard handling in a real browser via
    // `web-debug.sh` -- Ctrl+A/C/V in particular. Remove once that is settled.
    tree.branch(
        slot,
        TextInput::new()
            .text("select all me")
            .hint_text("ctrl+a here")
            .font_size(FontSize::new(16))
            .foreground(Color::gray(100))
            .background(Color::gray(800))
            .accent(Color::cyan(400))
            .rounding(Rounding::Sm)
            .outline(1)
            .at(Location::new().xs(
                10.0.pct().as_left().with(90.0.pct().as_right()),
                86.0.pct().as_top().with(34.px().as_height()),
            ))
            .elevate(Elevation::up(10)),
    );
}
