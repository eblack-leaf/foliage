//! The site as one value: what is on screen, and what each `Leaf` on it means.
//!
//! Nothing here is handed to the engine. The engine reports that *some* `Leaf` was clicked or
//! that *some* timer ran out; turning that back into "the rail's third entry" or "arm the
//! hero's fold hint" is this module's whole job, and it does it against plain Rust fields.

use foliage::{
    Bare, Bloom, Canopy, Elevation, Grid, GridExt, Grows, HrefLink, Leaf, Location, Sprout,
};

use crate::site;
use crate::site::drawer::{Controls, Drawer};
use crate::site::rail::Rail;
use crate::site::router;
use crate::site::{Grow, Page};

pub(crate) struct Site {
    /// The one parent every route's slot hangs off. Its own grid is what a slot's full-bleed
    /// `Location` resolves against.
    router: Leaf,
    /// Holds the rail. Lives outside the router's subtree, so switching sections tears down
    /// content without rebuilding the frame -- and gives the rail one parent to clear when the
    /// active entry changes.
    host: Leaf,
    controls: Controls,
    drawer: Drawer,
    /// Which route is showing. `None` only before the first build.
    route: Option<usize>,
    /// The slot holding it, so the next navigation knows what to tear down.
    slot: Option<Leaf>,
    rail: Rail,
    page: Page,
}

impl Site {
    /// Everything that outlives a route, plus the opening screen.
    pub(crate) fn grow(canopy: &mut Canopy) -> Self {
        let router = canopy.leaf(
            Bare::new()
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                // The route slot's own `Location` resolves against this: every scene is a
                // full-bleed child, so without a grid the first route spawned panics on a
                // parent that has none.
                .grid(Grid::default()),
        );
        let host = site::drawer::host(canopy);
        let controls = site::drawer::build(canopy);
        let mut site = Self {
            router,
            host,
            controls,
            drawer: Drawer::default(),
            route: None,
            slot: None,
            rail: Rail::default(),
            page: Page::default(),
        };
        // Route 0 is the hero. There is no separate initial-build path -- the opening screen
        // is just the first navigation.
        site.navigate(canopy, 0);
        site
    }

    /// Tears the current scene down and builds `index` in its place.
    ///
    /// The rail is respawned rather than diffed -- six labels and an indicator is cheaper to
    /// rebuild than to reconcile, and it keeps "which entry is active" a build-time input
    /// instead of live state to keep in sync.
    pub(crate) fn navigate(&mut self, canopy: &mut Canopy, index: usize) {
        let index = index.min(router::ROUTES.len() - 1);
        if self.route == Some(index) {
            return;
        }
        self.route = Some(index);
        if let Some(slot) = self.slot.take() {
            canopy.prune(slot);
        }
        if let Some(surface) = self.rail.surface.take() {
            canopy.prune(surface);
        }
        // Dropped with the scene it describes, so a click on a `Leaf` from the section you
        // just left cannot be answered by the one you are now in.
        self.page = Page::default();
        let slot = {
            let mut g = Grow::new(&mut *canopy, &mut self.page);
            router::build_route(&mut g, self.router, index)
        };
        self.slot = Some(slot);
        // route 0 is the hero and has no rail; sections start at 1
        let section = index.checked_sub(1);
        self.rail = site::rail::build(canopy, self.host, section);
        self.drawer
            .set_available(canopy, self.host, self.controls, section.is_some());
    }

    /// Answers one emission.
    ///
    /// Ordered by how permanent the target is: the drawer's controls outlive every route, the
    /// rail outlives the scene inside it, and the page's own targets are the ones that were
    /// built moments ago. A `Leaf` can only ever be in one of the three.
    pub(crate) fn respond(&mut self, canopy: &mut Canopy, bloom: Bloom) {
        match bloom {
            Bloom::Clicked(leaf) => self.clicked(canopy, leaf),
            // The `Leaf` names a timer and is spent, so it is taken out of the list rather
            // than left to be re-matched.
            Bloom::TimerFinished(timer) => {
                if let Some(at) = self.page.arming.iter().position(|(t, _)| *t == timer) {
                    let (_, target) = self.page.arming.swap_remove(at);
                    canopy.enable(target);
                }
            }
            _ => {}
        }
    }

    fn clicked(&mut self, canopy: &mut Canopy, leaf: Leaf) {
        if leaf == self.controls.backing {
            let open = !self.drawer.open;
            self.drawer.set_open(canopy, self.host, self.controls, open);
            return;
        }
        // tapping the content behind is the other half of the gesture, and the reason the
        // scrim exists at all
        if leaf == self.controls.scrim {
            self.drawer
                .set_open(canopy, self.host, self.controls, false);
            return;
        }
        if let Some(route) = self.rail.route_for(leaf) {
            self.navigate(canopy, route);
            return;
        }
        // Ahead of nav and links: a demo's own controls are the innermost thing on the page,
        // and a demo answering the click is the end of it.
        for demo in self.page.demos.iter_mut() {
            if demo.clicked(canopy, leaf) {
                return;
            }
        }
        let nav = self
            .page
            .nav
            .iter()
            .find(|(target, _)| *target == leaf)
            .map(|(_, route)| *route);
        if let Some(route) = nav {
            self.navigate(canopy, route);
            return;
        }
        let link = self
            .page
            .links
            .iter()
            .find(|(target, _)| *target == leaf)
            .map(|(_, href)| *href);
        if let Some(href) = link {
            HrefLink::new(href).navigate();
        }
    }

    /// Everything that has to be recomputed every frame, because every input it reads is
    /// something only this frame knows.
    pub(crate) fn tick(&mut self, canopy: &mut Canopy) {
        if let Some(readout) = self.page.readout.as_mut() {
            readout.drive(canopy);
        }
        for demo in self.page.demos.iter_mut() {
            demo.drive(canopy);
        }
        let dt = canopy.frame_time().as_secs_f32();
        for traverse in self.page.traverses.iter_mut() {
            traverse.drive(canopy, dt);
        }
    }
}
