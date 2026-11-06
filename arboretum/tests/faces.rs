//! Every face the apps bundle, held to what foliage holds a face to -- and to what the apps set in
//! it.
//!
//! foliage refuses a proportional face when it is registered, which is at boot, in a window, where
//! a test cannot see it. So the same probe is run here against the same bytes: the advances of the
//! characters spanning the narrowest and widest Latin shapes, which a monospaced face agrees on.
//!
//! And a face has no fallback. A character it lacks draws nothing at all, silently, so each face is
//! asked for every character the apps actually set in it.

use fontdue::{Font, FontSettings};

/// The characters foliage measures a face against (`foliage/src/text/font.rs`), at the size it
/// reads them at.
const PROBE: [char; 6] = ['i', 'W', 'm', '.', '1', 'g'];
const PX: f32 = 40.0;
const TOLERANCE: f32 = 0.01;

fn face(bytes: &[u8]) -> Font {
    Font::from_bytes(bytes, FontSettings::default()).expect("a face parses")
}

fn monospaced(name: &str, bytes: &[u8]) {
    let font = face(bytes);
    let advances: Vec<(char, f32)> = PROBE
        .iter()
        .filter(|&&c| font.lookup_glyph_index(c) != 0)
        .map(|&c| (c, font.metrics(c, PX).advance_width))
        .collect();
    assert!(advances.len() >= 2, "{name} carries too little of the probe to be measured");
    let (first, one) = advances[0];
    for &(c, advance) in &advances[1..] {
        assert!(
            (advance - one).abs() <= TOLERANCE,
            "{name} is not monospaced: '{first}' advances {one} but '{c}' advances {advance}"
        );
    }
}

fn covers(name: &str, bytes: &[u8], text: &str) {
    let font = face(bytes);
    let missing: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && font.lookup_glyph_index(*c) == 0)
        .collect();
    assert!(missing.is_empty(), "{name} has no glyph for {missing:?}");
}

const MAJOR: &[u8] =
    include_bytes!("../assets/fonts/major-mono-display/MajorMonoDisplay-Regular.ttf");
const XANH: &[u8] = include_bytes!("../assets/fonts/xanh-mono/XanhMono-Regular.ttf");
const XANH_ITALIC: &[u8] = include_bytes!("../assets/fonts/xanh-mono/XanhMono-Italic.ttf");
const PLEX: &[u8] = include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-Regular.ttf");
const PLEX_SEMI: &[u8] = include_bytes!("../assets/fonts/ibm-plex-mono/IBMPlexMono-SemiBold.ttf");

/// Printable ASCII, which every face is asked for.
fn ascii() -> String {
    (0x21u8..0x7f).map(char::from).collect()
}

#[test]
fn every_face_is_monospaced() {
    monospaced("Major Mono Display", MAJOR);
    monospaced("Xanh Mono", XANH);
    monospaced("Xanh Mono Italic", XANH_ITALIC);
    monospaced("IBM Plex Mono", PLEX);
    monospaced("IBM Plex Mono SemiBold", PLEX_SEMI);
}

#[test]
fn every_face_carries_ascii() {
    for (name, bytes) in [
        ("Major Mono Display", MAJOR),
        ("Xanh Mono", XANH),
        ("Xanh Mono Italic", XANH_ITALIC),
        ("IBM Plex Mono", PLEX),
        ("IBM Plex Mono SemiBold", PLEX_SEMI),
    ] {
        covers(name, bytes, &ascii());
    }
}

#[test]
#[ignore]
fn census() {
    let extras = "·…→←↑↓×–—σ✓●◆○□■★♪°±≈′″“”‘’€£¥§¶•→⟶↗↘";
    for (name, bytes) in [
        ("Major Mono Display", MAJOR),
        ("Xanh Mono", XANH),
        ("Xanh Mono Italic", XANH_ITALIC),
        ("IBM Plex Mono", PLEX),
        ("IBM Plex Mono SemiBold", PLEX_SEMI),
    ] {
        let font = face(bytes);
        let missing: String = extras.chars().filter(|c| font.lookup_glyph_index(*c) == 0).collect();
        println!("{name}: missing {missing}");
    }
    let jb = face(include_bytes!("../../foliage/src/text/JetBrainsMonoNL-Medium.ttf"));
    let missing: String = extras.chars().filter(|c| jb.lookup_glyph_index(*c) == 0).collect();
    println!("JetBrains Mono (bundled): missing {missing}");
}

/// The bundled face sets everything that is not a display line, and the apps' text leans on a few
/// marks beyond ASCII: the middle dot that separates every caption, quotes round a prompt, the
/// arrow of a walk, the ellipsis of a clipped line, a price.
#[test]
fn the_bundled_face_carries_the_marks_the_apps_set() {
    covers(
        "JetBrains Mono (bundled)",
        include_bytes!("../../foliage/src/text/JetBrainsMonoNL-Medium.ttf"),
        "·…→“”€×–—",
    );
}
