use crate::chrome;
use crate::home::home;
use crate::navigator;
use crate::next::next;
use crate::third::third;
use crate::toc::toc;
use foliage::{
    EcsExtension, Elevation, Entity, Foliage, GridExt, Location, Router, RouterRoutes, Sprout,
    Tree,
};

pub fn build(foliage: &mut Foliage) {
    let router = foliage.world.leaf(
        Router::new()
            .routes(RouterRoutes::new([
                home as fn(&mut Tree, Entity),
                toc as fn(&mut Tree, Entity),
                next as fn(&mut Tree, Entity),
                third as fn(&mut Tree, Entity),
            ]))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );

    // outside the router's subtree on purpose -- it survives every route switch and
    // drives navigation itself, rather than being torn down and rebuilt like scene
    // content.
    navigator::build(&mut foliage.world.commands(), router);
    // global, site-wide chrome (brand mark + github + home/ToC) -- also outside the
    // router's subtree, and elevated above the inner-site navigator so the two are always
    // visually distinct layers, never competing for the same space.
    chrome::build(&mut foliage.world.commands(), router);
}
