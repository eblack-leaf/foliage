use foliage::{
    bevy_ecs, EcsExtension, Event, Foliage, FontSize, Stem, Text, Tree, Trigger,
};

mod icon;
mod image;
#[derive(Event)]
pub(crate) struct Home {
    // args
    pub(crate) value: String,
}
impl Home {
    pub(crate) fn create(trigger: Trigger<Self>, mut tree: Tree) {
        // setup actions
        let id = tree.leaf(());
        // hook to update dependencies
        tree.write_to(id, Stem::some(trigger.entity()));
        // hook to pull text mut + update to given + cached glyphs and such + set size
        tree.write_to(id, Text::new(format!("hello {}", trigger.event().value)));
        // enable
        tree.enable(id);
        // disable
        tree.disable(id);
        // recursive despawn
        tree.remove(id);
    }
    pub(crate) fn new() -> Self {
        Self {
            // args to send to create?
            value: " world!".to_string(),
        }
    }
}
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.desktop_size((400, 600)); // window-size
    foliage.url("foliage"); // web-root
    foliage.define(Home::create); // task to trigger
    let root = foliage.leaf(()); // Stem => require Branch (Group)
    foliage.send_to(Home::new(), root); // trigger_targets
    foliage.send(Home::new()); // just trigger
    foliage.queue(Home::new()); // buffered event
    let leaf = foliage.leaf((
        Text::new("hello world!"),
        FontSize::new(14),
        Stem::some(root),
        // location,
    )); // add single node
    let button = foliage.leaf((
        // Button::new(),
        // ForegroundColor::RED,
        // BackgroundColor::BLUE,
        // ButtonText::new("example"),
        // ButtonIcon::new(IconHandle::Git),
        Stem::some(leaf),
    ));
    foliage.remove(root); // remove all from branch downwards in tree
    foliage.photosynthesize(); // run
}
