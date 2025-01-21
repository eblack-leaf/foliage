use foliage::{bevy_ecs, Animation, Attachment, EcsExtension, Elevation, Foliage, Grid, GridExt, InteractionListener, Location, Named, Stem};
use foliage::{Event, Res, Tree, Trigger};
impl Attachment for Usage {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Usage::init);
    }
}
#[derive(Event)]
pub(crate) struct Usage {}
impl Usage {
    pub(crate) fn init(trigger: Trigger<Self>, mut tree: Tree, named: Res<Named>) {
        let row_size = 40;
        let root = tree.leaf((
            Grid::new(12.col().gap(8), row_size.px().gap(8)),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            InteractionListener::new().scroll(true),
            Elevation::abs(0),
            Stem::none(),
        ));
        // TODO elements
        let seq = tree.sequence();
        let other = named.get("root");
        tree.animate(
            Animation::new(Location::new().xs(
                100.pct().left().with(200.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ))
                .start(0)
                .finish(1000)
                .targeting(other)
                .during(seq),
        );
    }
}
