//! The arithmetic under the look: cuts, crossings, facing, and what a chain wears.

use lichen::{Cut, Press, Silhouette, Turn, gate, words};

/// A square page with a square traced on it, corner to corner of its middle half.
fn square() -> Silhouette {
    Silhouette::traced(
        &[(0.25, 0.25), (0.75, 0.25), (0.75, 0.75), (0.25, 0.75)],
        (100.0, 100.0),
        Turn::None,
    )
}

fn close(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4
}

#[test]
fn a_vertical_cut_through_the_middle_crosses_top_and_bottom() {
    let shape = square();
    let cut = Cut::new((0.5, 0.0), (0.5, 1.0), (0.3, 0.5));
    let (a, b) = shape.crossings(cut).expect("the line crosses the square");
    // The square fills unit space: its left edge at 0, its right at 1.
    assert!(close(a, (0.5, 0.0)) || close(a, (0.5, 1.0)), "{a:?}");
    assert!(close(b, (0.5, 0.0)) || close(b, (0.5, 1.0)), "{b:?}");
}

#[test]
fn a_cut_keeps_the_side_it_was_told_to() {
    let shape = square();
    let left = shape.cut(Cut::new((0.5, 0.0), (0.5, 1.0), (0.3, 0.5)));
    let right = shape.cut(Cut::new((0.5, 0.0), (0.5, 1.0), (0.7, 0.5)));
    // Half the square each, and the same aspect as the whole, since a part keeps its space.
    assert_eq!(left.aspect(), shape.aspect());
    assert_eq!(right.aspect(), shape.aspect());
    assert!(close(
        shape.facing(Cut::new((0.5, 0.0), (0.5, 1.0), (0.3, 0.5))),
        (1.0, 0.0)
    ));
    assert!(close(
        shape.facing(Cut::new((0.5, 0.0), (0.5, 1.0), (0.7, 0.5))),
        (-1.0, 0.0)
    ));
}

#[test]
fn a_line_that_misses_the_shape_crosses_nothing() {
    let shape = square();
    assert!(
        shape
            .crossings(Cut::new((0.9, 0.0), (0.9, 1.0), (0.5, 0.5)))
            .is_none()
    );
}

#[test]
fn a_chain_arms_its_first_undone_step_and_holds_the_rest() {
    let wears: Vec<Press> = gate(&[true, true, false, false]).collect();
    assert_eq!(
        wears,
        [Press::Rest, Press::Rest, Press::Armed, Press::Inert]
    );
    let done: Vec<Press> = gate(&[true, true]).collect();
    assert_eq!(done, [Press::Rest, Press::Rest]);
}

#[test]
fn only_an_inert_press_is_out_of_reach() {
    for press in [Press::Rest, Press::Armed, Press::Chosen, Press::Danger] {
        assert!(press.within(), "{press:?}");
    }
    assert!(!Press::Inert.within());
}

#[test]
fn a_cut_word_ends_in_an_ellipsis_within_its_columns() {
    assert_eq!(words::cut("short", 10), "short");
    let cut = words::cut("a rather long name for a row", 10);
    assert!(cut.chars().count() <= 10);
    assert!(cut.ends_with('…'));
}
