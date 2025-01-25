use foliage::{
    bevy_ecs, Animation, Attachment, Ease, EcsExtension, Elevation, Foliage, Grid, GridExt,
    InteractionListener, Location, Named, Opacity, Stem,
};
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
                0.pct().left().with(0.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            InteractionListener::new().scroll(true),
            Elevation::abs(0),
            Stem::none(),
        ));
        // TODO elements
        // TODO back-to-home button => enable-interactive + cleanup usage-root
        // TODO (so can run again from home + memory usage)
        let seq = tree.sequence();
        let home = named.get("home");
        tree.animate(
            Animation::new(Location::new().xs(
                100.pct().left().with(200.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ))
            .start(0)
            .finish(1000)
            .targeting(home)
            .eased(Ease::ACCELERATE)
            .during(seq),
        );
        tree.animate(
            Animation::new(Opacity::new(0.0))
                .start(500)
                .finish(1000)
                .targeting(home)
                .during(seq),
        );
        tree.animate(
            Animation::new(Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ))
            .start(0)
            .finish(1000)
            .eased(Ease::ACCELERATE)
            .targeting(root)
            .during(seq),
        );
        // TODO anim in elements
    }
}
