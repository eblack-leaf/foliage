use crate::home::home;
use foliage::{
    EcsExtension, Elevation, Entity, Foliage, GridExt, Location, Router, RouterRoutes, Sprout,
    Tree,
};

pub fn build(foliage: &mut Foliage) {
    foliage.world.leaf(
        Router::new()
            .routes(RouterRoutes::new([home as fn(&mut Tree, Entity)]))
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
}
