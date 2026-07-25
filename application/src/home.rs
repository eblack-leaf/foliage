use crate::navigator::NavigatorLanded;
use crate::type_in;
use foliage::{EcsExtension, Entity, Tree, Trigger};

/// The polygon/lines/icon "navigator" moved out to its own persistent module -- it
/// survives route switches, so it can no longer live inside any one scene. This route's
/// own content is just the type-in effect now. It doesn't start at this scene's own
/// t=0 (which, for the very first route, is immediately at boot, before the navigator's
/// intro has even begun) -- it waits for `NavigatorLanded`, targeted at this exact
/// `slot`. Home is revisitable, so this subscription is re-registered fresh on every
/// visit (a new `slot` each time) -- the navigator resends the event on every later
/// return, not just the first landing.
pub fn home(tree: &mut Tree, slot: Entity) {
    tree.subscribe(slot, move |_: Trigger<NavigatorLanded>, mut tree: Tree| {
        let seq = tree.sequence();
        type_in::type_in(&mut tree, slot, seq, 0);
    });
}
