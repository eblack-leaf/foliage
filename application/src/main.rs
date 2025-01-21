#![allow(unused)]

use crate::home::Home;
use crate::usage::Usage;
use foliage::Foliage;

mod home;
mod usage;

fn main() {
    let mut foliage = Foliage::new();
    foliage.enable_tracing(
        tracing_subscriber::filter::Targets::new().with_target("foliage", tracing::Level::TRACE),
    );
    foliage.desktop_size((360, 800));
    foliage.url("foliage");
    foliage.attach::<Home>();
    foliage.attach::<Usage>();
    foliage.send(Home {});
    foliage.photosynthesize(); // run
}
