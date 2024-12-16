use foliage::{bevy_ecs, nalgebra, vector, Branch, Event, Foliage, Token, Tree, Trigger};

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
    foliage.tokens(
        Design::new()
            .system("on-secondary-container", "ref.orange-100")
    ); // define design-token values
    foliage.define(Home::create); // task to trigger
    let branch = foliage.branch(Branch::new(Home::new())); // predefined set of nodes (encapsulated)
    // Tokens are easy to pull from file as value-based config to signal to renderer
    let token = Token::new(); // design-token for style (color, size, ...) THEME => actual-value
    let leaf = foliage.leaf(Leaf::new((Text::new("hello world!"), token)).stem(branch)); // add single node
    foliage.remove(branch); // remove all from branch downwards in tree
    foliage.photosynthesize(); // run
}
