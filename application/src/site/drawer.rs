//! The rail's phone form: the same navigation, slid in from the left over the content.
//!
//! Not a separate widget -- it is the *same* rail entity, moved. `md`+ keeps it permanently
//! on screen and this module's controls are parked off-canvas; `xs` starts it hidden and
//! the menu button brings it in. One list, one set of handlers, two placements.

use foliage::{
    Animation, Color, Ease, Elevation, Entity, Grid, GridExt, Icon, IconId,
    InteractionListener, InteractionPropagation, Stem, Location, OnClick, Opacity, Panel, Query,
    Rounding, Sprout, Tree, Trigger, Visibility, component,
};

use crate::icons::IconHandles;
use crate::site::role;
use crate::site::shell::rail_host_location;

/// The drawer's whole state, on the rail host so it survives the rail being rebuilt on every
/// route change -- exactly when a captured `bool` would reset and leave the drawer open with
/// nothing in it.
///
/// Both flags live together because every control's visibility is a function of the pair,
/// and splitting them meant two writers racing to set the same `Visibility`.
#[component]
#[derive(Copy, Clone, Default)]
pub(crate) struct Drawer {
    pub(crate) open: bool,
    /// False on the hero, which has no rail -- so it has nothing to open, and none of these
    /// controls belong there.
    pub(crate) available: bool,
}

const MENU: i32 = 24;
const MENU_INSET: i32 = 14;
/// How far the backing extends past the glyph on every side.
const MENU_PAD: i32 = 10;
/// Short enough to feel like a direct response to the tap. The site's long entrance morph
/// is for arriving somewhere; this is just a panel moving.
const SLIDE_MS: u64 = 240;
/// How dark the content behind goes. Enough to say "this is on top of that" without
/// hiding it.
const SCRIM_OPACITY: f32 = 0.55;

/// The controls the drawer owns, so the caller can hide them where they do not belong.
#[derive(Copy, Clone)]
pub(crate) struct Controls {
    pub(crate) scrim: Entity,
    pub(crate) backing: Entity,
    pub(crate) menu: Entity,
}

/// Adds the scrim and menu button, and wires both to toggle `host`.
pub(crate) fn build(tree: &mut Tree, host: Entity) -> Controls {
    // Full-bleed, under the rail and over the content. Always present: toggling its
    // *interaction* is what makes it inert when closed, rather than spawning and despawning
    // it, so there is no rebuild to get wrong.
    let scrim = tree.leaf(
        Panel::new()
            .color(Color::stone(950))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(4))
            .with((Opacity::new(0.0), InteractionListener::new())),
    );
    tree.disable(scrim);

    // Both parked off-canvas at `md`+, where the rail is permanent and a menu button would
    // be a control for something already visible.
    let placement = |size: i32, inset: i32| {
        Location::new()
            .xs(
                inset.px().as_left().with(size.px().as_width()),
                inset.px().as_top().with(size.px().as_height()),
            )
            .md(
                (-(size + inset)).px().as_left().with(size.px().as_width()),
                inset.px().as_top().with(size.px().as_height()),
            )
    };
    // The backing is the button: it is the whole target, so a tap near the glyph rather than
    // exactly on it still lands. The icon passes through, or it would win the hit-test on
    // the pixels it covers and split one control into two.
    let backing = tree.leaf(
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::Sm)
            .at(placement(MENU + MENU_PAD * 2, MENU_INSET - MENU_PAD))
            .elevate(Elevation::up(6))
            .with(InteractionListener::new()),
    );
    let menu = tree.leaf(
        Icon::new(IconId::from(IconHandles::Menu))
            .color(role::on_surface())
            .at(placement(MENU, MENU_INSET))
            .elevate(Elevation::up(7))
            .with(InteractionPropagation::pass_through()),
    );

    let controls = Controls {
        scrim,
        backing,
        menu,
    };
    tree.on_click(
        backing,
        move |_: Trigger<OnClick>, drawers: Query<&Drawer>, mut tree: Tree| {
            let open = drawers.get(host).map(|d| d.open).unwrap_or(false);
            set_open(&mut tree, host, controls, &drawers, !open);
        },
    );
    // tapping the content behind is the other half of the gesture, and the reason the scrim
    // exists at all
    tree.on_click(
        scrim,
        move |_: Trigger<OnClick>, drawers: Query<&Drawer>, mut tree: Tree| {
            set_open(&mut tree, host, controls, &drawers, false);
        },
    );
    controls
}

/// Follows the route. Always lands closed, so returning to a section never arrives with the
/// drawer already over it.
pub(crate) fn set_available(tree: &mut Tree, host: Entity, controls: Controls, available: bool) {
    apply(
        tree,
        host,
        controls,
        Drawer {
            open: false,
            available,
        },
    );
}

/// Opens or closes, keeping whatever the route already decided about availability.
pub(crate) fn set_open(
    tree: &mut Tree,
    host: Entity,
    controls: Controls,
    drawers: &Query<&Drawer>,
    open: bool,
) {
    let available = drawers.get(host).map(|d| d.available).unwrap_or(false);
    apply(tree, host, controls, Drawer { open, available });
}

/// The single writer. Everything the drawer shows is derived from `state` here, so no two
/// callers can disagree about a control's visibility -- which is what made "does the hero
/// have a rail" depend on call order.
///
/// `available` dominates: on the hero nothing shows and the rail stays parked, whatever
/// `open` says.
fn apply(tree: &mut Tree, host: Entity, controls: Controls, state: Drawer) {
    tree.write_to(host, state);
    let showing = state.available && state.open;
    let scrim = controls.scrim;

    tree.write_to(scrim, Visibility::new(state.available));
    // The button hides while the rail is in: the scrim is the way out, and a menu button over
    // an open menu controls something already in front of you.
    //
    // Disabled as well as hidden. `Visibility` stops it drawing but not competing for input,
    // and its 44px target sits at the top-left corner -- exactly on top of the rail's own
    // "back to the hero" area once the rail slides in, and above it in elevation, so an
    // invisible button was swallowing that click.
    let controls_live = state.available && !state.open;
    for entity in [controls.backing, controls.menu] {
        tree.write_to(entity, Visibility::new(controls_live));
    }
    if controls_live {
        tree.enable(controls.backing);
    } else {
        tree.disable(controls.backing);
    }

    let seq = tree.sequence();
    // The target carries *both* breakpoints: animating a `Location` writes the whole
    // component, so a target with only `xs` would drop the `md` placement and the rail
    // would jump off-canvas on desktop the first time this ran.
    tree.animate(
        Animation::new(rail_host_location(showing))
            .targeting(host)
            .during(seq)
            .start(0)
            .finish(SLIDE_MS)
            .eased(Ease::EMPHASIS),
    );

    let fade = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(if showing { SCRIM_OPACITY } else { 0.0 }))
            .targeting(scrim)
            .during(fade)
            .start(0)
            .finish(SLIDE_MS)
            .eased(Ease::Linear),
    );
    if showing {
        tree.enable(scrim);
    } else {
        tree.disable(scrim);
    }
}

/// The rail host: sized to the rail's own footprint, carrying the drawer state and the grid
/// the rail surface resolves against.
pub(crate) fn host(tree: &mut Tree) -> Entity {
    tree.leaf(
        Stem::new()
            .at(rail_host_location(false))
            .elevate(Elevation::up(5))
            .with((Grid::new(1.col().gap(0), 1.row().gap(0)), Drawer::default())),
    )
}
