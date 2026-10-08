//! The glyph atlas, less its texture: which cuts are on the sheet, where, and what happens when it
//! runs out of room.
//!
//! The texture is the platform layer the suite cannot reach, and what is written to it is only ever
//! what is decided here -- so this is the whole of the atlas that can be wrong without a device.
//! The ink is the bundled face's own, rasterised for real.

use crate::ash::atlas::{Cut, Cuts, Placed, Shelves};
use crate::text::Font;
use crate::text::font::Fonts;

/// A character of the bundled face at the default size, at a density of one.
fn cut(character: char) -> Cut {
    Cut::new(Font::DEFAULT, 16, 1.0, character)
}

/// Where each of `characters` is placed, in order, and whether each was packed just now.
fn place(cuts: &mut Cuts, fonts: &Fonts, characters: &str) -> Vec<(Placed, bool)> {
    characters
        .chars()
        .map(|character| {
            let (placed, packed) = cuts.place(fonts, cut(character));
            (placed, packed.is_some())
        })
        .collect()
}

// Shelves.

/// A row is filled left to right with a gutter between, and the next starts below the tallest
/// bitmap in it.
#[test]
fn shelves_fill_across_and_then_down() {
    let mut shelves = Shelves::new(32);
    assert_eq!(shelves.pack(10, 8), Some((0, 0)));
    assert_eq!(shelves.pack(10, 6), Some((11, 0)));
    assert_eq!(shelves.pack(10, 8), Some((22, 0)));
    assert_eq!(shelves.pack(10, 8), Some((0, 9)));
}

/// Past the last shelf there is no room, and a bitmap larger than the sheet never had any. Cleared,
/// the sheet starts again at its corner.
#[test]
fn a_full_sheet_has_no_room_until_it_is_cleared() {
    let mut shelves = Shelves::new(32);
    assert_eq!(shelves.pack(40, 4), None);
    assert_eq!(shelves.pack(32, 20), Some((0, 0)));
    assert_eq!(shelves.pack(32, 20), None);
    shelves.clear();
    assert_eq!(shelves.pack(32, 20), Some((0, 0)));
}

// Cuts.

/// A cut is rasterised and packed the first time it is asked for, and every time after is the place
/// it was given.
#[test]
fn a_cut_is_packed_once() {
    let fonts = Fonts::new();
    let mut cuts = Cuts::new(64);
    let first = place(&mut cuts, &fonts, "a");
    let again = place(&mut cuts, &fonts, "a");
    assert!(first[0].1);
    assert!(!again[0].1);
    assert_eq!(first[0].0, again[0].0);
    assert!(first[0].0.area.width > 0.0);
}

/// The same character at another density is another cut, because a bitmap is made in device
/// pixels.
#[test]
fn a_density_is_part_of_the_cut() {
    let fonts = Fonts::new();
    let mut cuts = Cuts::new(64);
    let (one, _) = cuts.place(&fonts, Cut::new(Font::DEFAULT, 16, 1.0, 'a'));
    let (two, packed) = cuts.place(&fonts, Cut::new(Font::DEFAULT, 16, 2.0, 'a'));
    // Packed beside the first rather than found as it, and with ink of its own: cut at twice the
    // pixels, its bounds round differently.
    assert!(packed.is_some());
    assert_ne!(one.uv, two.uv);
}

/// A character with no ink takes no room, and is not counted as running out of it.
#[test]
fn a_space_is_held_as_nothing() {
    let fonts = Fonts::new();
    let mut cuts = Cuts::new(64);
    let space = place(&mut cuts, &fonts, " ");
    assert!(!space[0].1);
    assert_eq!(space[0].0.area.width, 0.0);
    assert!(!cuts.overflowed());
}

/// A cut that finds no room is held as nothing for as long as the sheet stands, and says the sheet
/// overflowed. Cut again, the sheet has room for it, at its corner.
#[test]
fn a_cut_with_no_room_waits_for_the_sheet_to_be_cut_again() {
    let fonts = Fonts::new();
    let mut cuts = Cuts::new(32);
    // Six glyphs of the default size fill a sheet this small.
    let placed = place(&mut cuts, &fonts, "abcdefghij");
    assert!(cuts.overflowed());
    let refused = placed
        .iter()
        .position(|(placed, _)| *placed == Placed::default());
    let refused = refused.expect("one found no room");
    let character = "abcdefghij".chars().nth(refused).expect("a character");
    // Asked for again, it is what it was: nothing, and nothing rasterised for it.
    let (again, packed) = cuts.place(&fonts, cut(character));
    assert_eq!(again, Placed::default());
    assert!(packed.is_none());

    cuts.clear();
    assert!(!cuts.overflowed());
    let (fresh, packed) = cuts.place(&fonts, cut(character));
    assert!(packed.is_some());
    assert_eq!((fresh.uv[0], fresh.uv[1]), (0.0, 0.0));
}

/// Full is a statement about what the frame draws: it is only found when the sheet has been cut
/// again from that and it still did not fit -- and a later time it fits says it is not full any
/// more.
#[test]
fn the_sheet_is_full_only_when_what_the_frame_draws_does_not_fit() {
    let fonts = Fonts::new();
    let mut cuts = Cuts::new(32);
    place(&mut cuts, &fonts, "abcdefghij");
    assert!(cuts.overflowed());
    assert!(!cuts.full());

    // Cut again from a frame that draws two characters: they fit, and it is not full.
    cuts.clear();
    place(&mut cuts, &fonts, "ab");
    cuts.settle();
    assert!(!cuts.full());
    assert!(!cuts.overflowed());

    // From a frame that draws ten: they do not, and it is.
    cuts.clear();
    place(&mut cuts, &fonts, "abcdefghij");
    cuts.settle();
    assert!(cuts.full());
    assert!(!cuts.overflowed());

    cuts.clear();
    place(&mut cuts, &fonts, "ab");
    cuts.settle();
    assert!(!cuts.full());
}

/// Forgotten, as for a new density, the sheet is empty and has never been full.
#[test]
fn a_forgotten_sheet_is_new() {
    let fonts = Fonts::new();
    let mut cuts = Cuts::new(32);
    cuts.clear();
    place(&mut cuts, &fonts, "abcdefghij");
    cuts.settle();
    assert!(cuts.full());

    cuts.forget();
    assert!(!cuts.full());
    let placed = place(&mut cuts, &fonts, "a");
    assert!(placed[0].1);
    assert_eq!((placed[0].0.uv[0], placed[0].0.uv[1]), (0.0, 0.0));
}
