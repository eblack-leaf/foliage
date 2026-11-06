//! Every style, painted and written out as a PNG to be looked at.
//!
//! Ignored by default -- it writes files rather than checking anything:
//!
//! ```sh
//! ARBORETUM_PREVIEW=/tmp/preview cargo test -p arboretum --test preview -- --ignored
//! ```

use arboretum::genart::{Denoise, Params, Ramp, Seed, Style, paint, sprite};
use foliage::Color;
use std::path::PathBuf;

fn out() -> PathBuf {
    let dir = std::env::var("ARBORETUM_PREVIEW").unwrap_or_else(|_| "target/preview".into());
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("a preview directory");
    dir
}

fn save(name: &str, pixels: Vec<u8>, width: u32, height: u32) {
    let image = image::RgbaImage::from_raw(width, height, pixels).expect("pixels fit the size");
    image.save(out().join(format!("{name}.png"))).expect("a preview is written");
}

#[test]
#[ignore]
fn styles() {
    let ramps = [
        Ramp::new(&[
            Color::oklch(0.16, 0.03, 300.0),
            Color::oklch(0.42, 0.14, 305.0),
            Color::oklch(0.72, 0.14, 330.0),
            Color::oklch(0.95, 0.04, 80.0),
        ]),
        Ramp::new(&[
            Color::oklch(0.18, 0.04, 160.0),
            Color::oklch(0.45, 0.11, 150.0),
            Color::oklch(0.78, 0.13, 120.0),
            Color::oklch(0.96, 0.05, 95.0),
        ]),
    ];
    for (n, style) in Style::ALL.into_iter().enumerate() {
        let ramp = &ramps[n % 2];
        let params = Params::of(style, Seed::new(0x5eed + n as u64));
        save(style.name(), paint(style, params, ramp, 256, 256, 1.5), 256, 256);
    }
    let params = Params::of(Style::Bloom, Seed::new(0x5eed));
    let picture = paint(Style::Bloom, params, &ramps[0], 256, 256, 0.0);
    let denoise = Denoise::new(picture, 256, 256, Seed::new(1));
    for step in [0, 6, 12, 18, 24] {
        save(&format!("denoise-{step:02}"), denoise.frame(step, 24), 256, 256);
    }
    save("falloff", sprite::falloff(128, Color::oklch(0.8, 0.12, 305.0)), 128, 128);
}
