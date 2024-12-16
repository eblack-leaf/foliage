use foliage::{bevy_ecs, nalgebra, vector, Event, Foliage, Stem, Tree, Trigger};

mod icon;
mod image;
#[derive(Event)]
pub(crate) struct Home {
    // args
}
impl Home {
    pub(crate) fn create(trigger: Trigger<Self>, mut tree: Tree) {
        // setup actions
    }
    pub(crate) fn new() -> Self {
        Self {
            // args to send to create?
        }
    }
}
fn main() {
    let mut foliage = Foliage::new(); // library-handle
    foliage.desktop_size(vector![400, 600]); // window-size
    foliage.url("foliage"); // web-root
    foliage.define(Home::create); // task to trigger
    let root = foliage.leaf(Stem::none()); // Stem => require Branch (Group)
    foliage.send_to(root, Home::new()); // trigger_targets
    foliage.send(Home::new()); // just trigger
    let leaf = foliage.leaf((
        Text::new("hello world!"),
        FontSize::new(14),
        Stem::some(root), /* location */
    )); // add single node
    let button = foliage.leaf((
        Button::new(),
        ForegroundColor::RED,
        BackgroundColor::BLUE,
        ButtonText::new("example"),
        ButtonIcon::new(IconHandle::Git),
        Stem::some(leaf),
    ));
    foliage.flush([leaf, button]); // EvaluateCore::recursive() as event? or component-hook
    foliage.remove(branch); // remove all from branch downwards in tree
    foliage.photosynthesize(); // run
}
