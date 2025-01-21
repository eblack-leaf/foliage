use foliage::{bevy_ecs, Animation, Attachment, EcsExtension, Foliage, GridExt, Location, Named};
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
        let other = named.get("root");
        let seq = tree.sequence();
        tree.animate(
            seq,
            Animation::new(Location::new().xs(
                100.pct().left().with(200.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ))
            .start(0)
            .finish(500)
            .targeting(other),
        );
    }
}
