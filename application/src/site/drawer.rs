//! The rail's phone form: the same navigation, slid in from the left over the content.
//!
//! Not a separate widget -- it is the *same* rail host, moved. `md`+ keeps it permanently
//! on screen and this module's controls are parked off-canvas; `xs` starts it hidden and
//! the menu button brings it in. One list, one set of handlers, two placements.

use foliage::{
    Bare, Forest, Color, Ease, Elevation, Grid, GridExt, Grows, Icon, IconId, Leaf, Location,
    Motion, Panel, Rounding, Sprout,
};

use crate::icons::IconHandles;
use crate::site::role;
use crate::site::shell::rail_host_location;
use crate::site::timing;

/// The drawer's whole state. Both flags live together because every control's visibility is a
/// function of the pair, and splitting them meant two writers racing to set the same
/// visibility.
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
/// The button's whole footprint measured from the top-left corner -- the backing's inset plus
/// its size.
///
/// Public because the button floats *over* the content at `xs`, so a page has to know how much
/// of its own top-left corner is already spoken for. See [`site::PAGE_TOP`](crate::site::PAGE_TOP).
pub(crate) const MENU_FOOTPRINT: i32 = (MENU_INSET - MENU_PAD) + MENU + MENU_PAD * 2;
/// Short enough to feel like a direct response to the tap. The site's long entrance morph
/// is for arriving somewhere; this is just a panel moving.
const SLIDE_MS: u64 = 240;
/// How dark the content behind goes. Enough to say "this is on top of that" without
/// hiding it.
const SCRIM_OPACITY: f32 = 0.55;

/// The controls the drawer owns, so the caller can hide them where they do not belong.
#[derive(Copy, Clone)]
pub(crate) struct Controls {
    pub(crate) scrim: Leaf,
    pub(crate) backing: Leaf,
    pub(crate) menu: Leaf,
}

/// Adds the scrim and the menu button. Both are permanent: what makes them inert where they
/// do not belong is [`Drawer::apply`], not spawning and despawning, so there is no rebuild to
/// get wrong.
pub(crate) fn build(forest: &mut Forest) -> Controls {
    // Full-bleed, under the rail and over the content.
    let scrim = forest.leaf(
        Panel::new()
            .color(Color::stone(950))
            .rounding(Rounding::None)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(4))
            .opacity(0.0)
            .interactive(),
    );
    forest.disable(scrim);

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
    let backing = forest.leaf(
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::Sm)
            .at(placement(MENU + MENU_PAD * 2, MENU_INSET - MENU_PAD))
            .elevate(Elevation::up(6))
            .interactive(),
    );
    let menu = forest.leaf(
        Icon::new(IconId::from(IconHandles::Menu))
            .color(role::on_surface())
            .at(placement(MENU, MENU_INSET))
            .elevate(Elevation::up(7))
            .pass_through(),
    );

    Controls {
        scrim,
        backing,
        menu,
    }
}

impl Drawer {
    /// Follows the route. Always lands closed, so returning to a section never arrives with
    /// the drawer already over it.
    pub(crate) fn set_available(
        &mut self,
        forest: &mut Forest,
        host: Leaf,
        controls: Controls,
        available: bool,
    ) {
        self.open = false;
        self.available = available;
        self.apply(forest, host, controls);
    }
    /// Opens or closes, keeping whatever the route already decided about availability.
    pub(crate) fn set_open(
        &mut self,
        forest: &mut Forest,
        host: Leaf,
        controls: Controls,
        open: bool,
    ) {
        self.open = open;
        self.apply(forest, host, controls);
    }
    /// The single writer. Everything the drawer shows is derived from the pair of flags here,
    /// so no two callers can disagree about a control's visibility -- which is what made "does
    /// the hero have a rail" depend on call order.
    ///
    /// `available` dominates: on the hero nothing shows and the rail stays parked, whatever
    /// `open` says.
    fn apply(&self, forest: &mut Forest, host: Leaf, controls: Controls) {
        let showing = self.available && self.open;
        let scrim = controls.scrim;

        forest.visible(scrim, self.available);
        // The button hides while the rail is in: the scrim is the way out, and a menu button
        // over an open menu controls something already in front of you.
        //
        // Disabled as well as hidden. Hiding stops it drawing but not competing for input, and
        // its 44px target sits at the top-left corner -- exactly on top of the rail's own "back
        // to the hero" area once the rail slides in, and above it in elevation, so an invisible
        // button was swallowing that click.
        let controls_live = self.available && !self.open;
        for leaf in [controls.backing, controls.menu] {
            forest.visible(leaf, controls_live);
        }
        if controls_live {
            forest.enable(controls.backing);
        } else {
            forest.disable(controls.backing);
        }

        // The target carries *both* breakpoints: animating a `Location` writes the whole
        // value, so a target with only `xs` would drop the `md` placement and the rail would
        // jump off-canvas on desktop the first time this ran.
        forest.animate(
            host,
            Motion::Location(rail_host_location(showing)),
            timing(0, SLIDE_MS, Ease::EMPHASIS),
        );
        forest.animate(
            scrim,
            Motion::Opacity(if showing { SCRIM_OPACITY } else { 0.0 }),
            timing(0, SLIDE_MS, Ease::Linear),
        );
        if showing {
            forest.enable(scrim);
        } else {
            forest.disable(scrim);
        }
    }
}

/// The rail host: sized to the rail's own footprint, carrying the grid the rail surface
/// resolves against.
///
/// Sized to the rail rather than to the whole screen -- a full-screen host sat on top of the
/// content and grabbed interaction by default, so every click died in it.
pub(crate) fn host(forest: &mut Forest) -> Leaf {
    forest.leaf(
        Bare::new()
            .at(rail_host_location(false))
            .elevate(Elevation::up(5))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0))),
    )
}
