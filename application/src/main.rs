use foliage::action::{Actionable, ElmHandle};
use foliage::Foliage;
#[derive(Clone)]
struct Stuff {}
impl Actionable for Stuff {
    fn apply(self, mut handle: ElmHandle) {
        handle.add_element("first", |e| e.with_attr(()));
    }
}
fn main() {
    let mut foliage = Foliage::new();
    foliage.set_desktop_size((360, 800));
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.set_base_url("");
    foliage.enable_action::<Stuff>();
    foliage.run();
}
