use foliage::action::{Actionable, ElmConnection};
use foliage::Foliage;
#[derive(Clone)]
struct Stuff {}
impl Actionable for Stuff {
    fn apply(self, conn: ElmConnection) {
        todo!()
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
