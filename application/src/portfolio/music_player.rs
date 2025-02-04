use foliage::{bevy_ecs, Attachment, EcsExtension, Elevation, Event, Foliage, Grid, GridExt, Location, Panel, Stem, Tree, Trigger};

#[derive(Event)]
pub(crate) struct MusicPlayer {}
impl Attachment for MusicPlayer {
    fn attach(foliage: &mut Foliage) {
        foliage.define(MusicPlayer::init);
    }
}
impl MusicPlayer {
    pub(crate) fn init(trigger: Trigger<Self>, mut tree: Tree) {
        let root = tree.leaf((
            Panel::new(),
            Stem::none(),
            Elevation::abs(0),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Grid::new(12.col().gap(8), 40.px().gap(8)),
        ));
    }
}
