//! arboretum -- a portfolio grown on foliage and dressed in lichen.
//!
//! Three showpieces, one binary each, every one written against nothing but the public surfaces of
//! `foliage` and `lichen`:
//!
//! | Binary | What it is |
//! |---|---|
//! | `conservatory` | *latent flora*: an exhibition of AI-grown works, developed in front of you |
//! | `orchard` | *bits & bytes*: a boutique of digital goods, from the aisle to the receipt |
//! | `trellis` | one planning sheet: the people, the meetings and the specs of a launch |
//!
//! What they share lives here:
//!
//! - [`sprout`] -- parts in lichen's manner that lichen does not have yet: hover, a lean toward
//!   the cursor, soft light and shadow, an odometer, a typewriter, wires, a drawer, a courier that
//!   stands in for a server. Candidates, each one, for lichen itself.
//! - [`genart`] -- pictures made from seeds: the works, the shaders, the light and the shadow.
//!   Pure arithmetic over bytes, and tested as such.
//! - [`parts`] -- the layout helpers every page is built from.
//!
//! None of them reaches a server. Every app has an `api` module standing where one would, and
//! every answer it gives is carried through a [`sprout::Courier`], which is what makes a mock
//! *flow*: the wait, the failure and the confirmation, rather than an answer that was always there.

pub mod genart;
pub mod parts;
pub mod sprout;

use foliage::{Area, Foliage, Root};

/// Where the portfolio is published: the GitHub Pages project page, under `portfolio/`.
pub const PUBLISHED: &str = "https://eblack-leaf.github.io/foliage/portfolio/";

/// The path every app is served under on the web, from the site's root.
pub const PORTFOLIO: &str = "/foliage/portfolio/";

/// Builds the engine, names the app, and grows `R` in it.
///
/// `tune` is handed the engine before it runs, for whatever an app decides globally -- a drag that
/// is claimed sooner, say. Only construction differs per platform, and this is all of it.
pub fn boot<R: Root>(name: &str, size: Area, tune: impl FnOnce(&mut Foliage)) {
    trace();
    let mut foliage = Foliage::new();
    foliage.title(name);
    foliage.app_id(format!("arboretum-{name}"));
    foliage.desktop_size(size);
    tune(&mut foliage);
    foliage.root::<R>();
    foliage.photosynthesize();
}

/// Sends the engine's trace where it can be read, at whatever `RUST_LOG` asks for -- nothing by
/// default, and the console in a browser, which has no stderr and no clock to ask.
fn trace() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("off"));
    #[cfg(not(target_family = "wasm"))]
    {
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }
    #[cfg(target_family = "wasm")]
    {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .without_time()
                    .with_writer(tracing_web::MakeWebConsoleWriter::new()),
            )
            .try_init();
    }
}
