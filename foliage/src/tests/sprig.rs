//! B11: that an op issued off the frame is the same op, and that a thread which cannot sample can
//! still read.

use crate::coordinate::{Area, Section};
use crate::elevation::ResolvedElevation;
use crate::grove::Grove;
use crate::leaf::{Leaf, Presence};
use crate::sprig::Sprig;
use crate::tests::assets::{frame, png};
use crate::tests::{Observer, grove, resize, section, tick, tick_with};
use crate::{
    Boxed, FontSize, Grow, Location, Palette, Panel, Place, Sap, Source, Stem, Text, Timing, Vein,
    content, left, top,
};
use crate::text::Font;

/// A ten-pixel box at `x`, which is enough to tell two placements apart.
fn box_at(x: f32) -> Location {
    Location::new().xs(left(x.px()).width(10.px()), top(0.px()).height(10.px()))
}

/// The same sequence of ops, issued into whichever side is handed to it.
///
/// Written once and run twice, because F1 is a claim about *identical code*: a script that had to be
/// spelled differently on either side would be evidence of nothing.
fn script<G: Grow>(sink: &mut G) -> [Leaf; 4] {
    let trunk = sink.plant(Stem::new().at(box_at(0.0)));
    let first = sink.branch(trunk, Panel::new().at(box_at(10.0)));
    let second = sink.branch(trunk, Panel::new().at(box_at(20.0)));
    sink.color(first, Palette::Accent);
    sink.prune(second);
    let third = sink.branch(trunk, Panel::new().at(box_at(30.0)));
    [trunk, first, second, third]
}

/// Everything about the tree these leaves can be asked, identity aside -- which is what differs
/// between two groves and what nothing is claimed about.
fn state(grove: &Grove, leaves: &[Leaf]) -> Vec<(Presence, Option<Sap>, Option<Sap>, Option<Sap>)> {
    leaves
        .iter()
        .map(|leaf| {
            (
                grove.presence(*leaf),
                grove.tap(*leaf, Vein::Drawn),
                grove.tap(*leaf, Vein::Color),
                grove.tap(*leaf, Vein::Trunk).map(|trunk| match trunk {
                    // The name itself is not comparable across groves; whether there is one is.
                    Sap::Leaf(under) => Sap::Visible(under.is_some()),
                    other => other,
                }),
            )
        })
        .collect()
}

fn rank(grove: &Grove, leaf: Leaf) -> ResolvedElevation {
    grove.tree.rank(leaf)
}

/// F1's proof obligation: for any op sequence, the resulting state is identical whether it was
/// issued through `Grove` or through `Sprig`, given the same arrival order.
#[test]
fn a_script_produces_the_same_state_through_either_side() {
    let mut inside = grove();
    let mut outside = grove();
    let mut sprig = outside.sprig();

    let issued = script(&mut inside);
    let pushed = script(&mut sprig);
    tick(&mut inside);
    tick(&mut outside);

    assert_eq!(state(&inside, &issued), state(&outside, &pushed));
}

/// The elevation tie-break is allocation order, and a name taken off the frame takes its place in
/// that order where it was taken. Two elements level with each other are separated by which was
/// named first and by nothing about which side named it.
#[test]
fn a_name_taken_off_the_frame_holds_its_place_in_the_order() {
    let mut grove = grove();
    let mut sprig = grove.sprig();

    let first = sprig.plant(Stem::new().at(box_at(0.0)));
    let second = grove.plant(Stem::new().at(box_at(10.0)));
    let third = sprig.plant(Stem::new().at(box_at(20.0)));
    tick(&mut grove);

    assert_eq!(rank(&grove, first).stack, rank(&grove, second).stack);
    assert!(rank(&grove, second) > rank(&grove, first));
    assert!(rank(&grove, third) > rank(&grove, second));
}

/// Position in the queue is the whole of what decides, and which side an op came from is no part of
/// it. The same two writes in either order leave the element at whichever arrived last.
#[test]
fn arrival_order_is_the_whole_of_the_order() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = grove.plant(Stem::new().at(box_at(0.0)));
    tick(&mut grove);

    sprig.at(leaf, box_at(10.0));
    grove.at(leaf, box_at(20.0));
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(20.0, 0.0, 30.0, 10.0)
    );

    grove.at(leaf, box_at(30.0));
    sprig.at(leaf, box_at(40.0));
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(40.0, 0.0, 50.0, 10.0)
    );
}

/// The one observable difference concurrency leaves: an op lands in the drain of the frame it
/// arrived before, and in the next frame's otherwise. Nothing about how it is applied differs.
#[test]
fn an_op_lands_in_the_frame_it_arrived_before() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));

    tick(&mut grove);
    assert_eq!(grove.presence(leaf), Presence::Live);

    sprig.at(leaf, box_at(10.0));
    // Written between two frames, so the frame that has already run cannot have seen it.
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(0.0, 0.0, 10.0, 10.0)
    );
    tick(&mut grove);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(10.0, 0.0, 20.0, 10.0)
    );
}

/// An app that pushes through a handle instead of through the `Grove` it was lent. Both reach the
/// same queue, so the write lands in this frame's drain either way.
struct Worker {
    sprig: Sprig,
    leaf: Option<Leaf>,
}

impl crate::root::Rooted for Worker {
    fn frame(&mut self, _grove: &mut Grove, _pollen: crate::Pollen) {
        let leaf = *self
            .leaf
            .get_or_insert_with(|| self.sprig.plant(Stem::new().at(box_at(0.0))));
        self.sprig.at(leaf, box_at(50.0));
    }
}

/// A handle used from inside the frame is the frame: step 3 is before step 4, so what it wrote is
/// drained in the frame that wrote it, exactly as `Grove` would have been.
#[test]
fn a_handle_used_inside_the_frame_lands_in_that_frame() {
    let mut grove = grove();
    let mut app = Worker {
        sprig: grove.sprig(),
        leaf: None,
    };

    tick_with(&mut grove, &mut app);
    let leaf = app.leaf.expect("a frame has run");
    assert_eq!(grove.presence(leaf), Presence::Live);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(50.0, 0.0, 60.0, 10.0)
    );
}

/// The bound the whole type exists for, asserted rather than assumed.
#[test]
fn a_handle_crosses_threads() {
    fn carried<T: Send + Sync + Clone>() {}
    carried::<Sprig>();

    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = std::thread::spawn(move || sprig.plant(Stem::new().at(box_at(70.0))))
        .join()
        .expect("the thread");
    tick(&mut grove);

    assert_eq!(grove.presence(leaf), Presence::Live);
    assert_eq!(
        section(&grove, leaf),
        Section::from_edges(70.0, 0.0, 80.0, 10.0)
    );
}

/// Names are one sequence however many sides are drawing from it, so nothing taken off the frame can
/// collide with what the frame took.
#[test]
fn names_are_one_sequence() {
    let mut grove = grove();
    let mut sprig = grove.sprig();

    assert_ne!(sprig.plate(), grove.plate());
    assert_ne!(sprig.tween(0.0, 1.0, Timing::ms(1)), grove.timer(Timing::ms(1)));
    assert_ne!(sprig.sequence(), grove.sequence());
}

/// A picture named where the registry is out of reach: the name is valid at once, and the registry
/// grows to meet it when the pixels arrive.
#[test]
fn a_picture_can_be_named_and_filled_off_the_frame() {
    let mut grove = grove();
    let mut sprig = grove.sprig();

    let plate = sprig.plate();
    let element = grove.plant(crate::Image::new(plate).at(box_at(0.0)));
    tick(&mut grove);
    assert_eq!(grove.plates.size(plate), None);
    assert_eq!(grove.tap(element, Vein::Picture), Some(Sap::Picture(plate)));

    sprig.load(plate, vec![255; 4], Area::new(1.0, 1.0));
    tick(&mut grove);
    assert_eq!(grove.plates.size(plate), Some(Area::new(1.0, 1.0)));
}

/// A face registered off the frame is parsed and stored: `loaded` is reported only where the face
/// was filled into the registry the frame composes from.
///
/// That report is the whole of the proof available here, and deliberately: the only typeface in the
/// repository is the bundled one, and an unfilled name is *measured* as the bundled face by design --
/// so nothing about a run's metrics could tell a filled slot from a fallback. What is asserted
/// besides is that the name is usable at a callsite and is a name of its own.
#[test]
fn a_face_registered_off_the_frame_is_filled() {
    let bytes = include_bytes!("../text/JetBrainsMonoNL-Medium.ttf").to_vec();
    let mut grove = grove();
    let mut sprig = grove.sprig();

    let outside = sprig.font(bytes.clone());
    // The name is valid at once, and the bytes land at the next drain.
    let run = grove.plant(
        Text::new("hello")
            .font(outside)
            .font_size(FontSize::new().xs(16))
            .at(Location::new().xs(
                left(0.px()).width(content()),
                top(0.px()).height(content()),
            )),
    );
    tick(&mut grove);
    assert!(frame(&mut grove).loaded(outside));
    assert_eq!(section(&grove, run).area.height, 22.0);

    // One counter across the boundary, and neither side can ever be handed the bundled face's name.
    let inside = grove.font(bytes);
    assert_ne!(outside, inside);
    assert_ne!(outside, Font::DEFAULT);
    assert_ne!(inside, Font::DEFAULT);
}

/// The one thing that differs. Bytes written at a callsite are a statement the program made and a
/// proportional face there is a panic; bytes a worker holds are not, and a thread that panicked
/// would take the report with it -- so it is refused and told, as a fetched one is.
#[test]
fn bytes_a_worker_holds_are_refused_rather_than_panicked_on() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let font = sprig.font(vec![0; 16]);
    tick(&mut grove);

    assert!(frame(&mut grove).missing(font));
    // The name stays valid and unfilled, so what composes in it composes in the bundled face.
    let run = grove.plant(Text::new("hello").font(font).font_size(FontSize::new().xs(16)));
    tick(&mut grove);
    assert_eq!(grove.presence(run), Presence::Live);
}

/// A mark registered off the frame reaches the batch, and one that is not what it was said to be is
/// refused where the same bytes at a callsite would have been an assertion.
#[test]
fn a_mark_registered_off_the_frame_draws() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let field = sprig.icon(vec![255; 4 * 4 * 4], 4, 2.0);
    let refused = sprig.icon(vec![255; 16], 8, 2.0);
    let leaf = grove.plant(crate::Icon::new(field).at(box_at(0.0)));
    tick(&mut grove);

    let heard = frame(&mut grove);
    assert!(heard.loaded(field));
    assert!(heard.missing(refused));
    assert_eq!(grove.elm.icons.len(), 1);
    assert_eq!(grove.tap(leaf, Vein::Mark), Some(Sap::Mark(field)));
}

/// An encoded picture is decoded where every arrival is decoded, so a worker that fetched or built
/// one hands over the bytes rather than the pixels.
#[test]
fn an_encoded_picture_is_decoded_off_the_frame() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let plate = sprig.image(png(3, 2));
    tick(&mut grove);

    assert!(frame(&mut grove).loaded(plate));
    assert_eq!(grove.plates.size(plate), Some(Area::new(3.0, 2.0)));
}

/// Nothing is collected until someone asks, so a handle that only ever writes never accumulates a
/// report it will not read.
#[test]
fn a_handle_that_has_never_listened_holds_nothing() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));
    tick(&mut grove);
    sprig.prune(leaf);
    tick(&mut grove);
    tick(&mut grove);

    // The first call arms delivery and answers with nothing, whatever has already happened.
    assert!(sprig.pollen().is_empty());
}

/// The same value the app is handed, at the same point in the frame.
#[test]
fn a_report_reaches_a_listening_handle() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let mut app = Observer::default();
    assert!(sprig.pollen().is_empty());

    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));
    tick_with(&mut grove, &mut app);
    sprig.prune(leaf);
    // The prune is drained here and reported at step 3 of the frame after it.
    tick_with(&mut grove, &mut app);
    tick_with(&mut grove, &mut app);

    assert!(app.last().withered(leaf));
    let reports = sprig.pollen();
    assert!(reports.iter().any(|pollen| pollen.withered(leaf)));
    // And what was taken is not handed out twice.
    assert!(sprig.pollen().is_empty());
}

/// A frame that reported nothing delivers nothing, so an idle engine does not fill an inbox with
/// empty reports.
#[test]
fn an_idle_frame_delivers_no_report() {
    let mut grove = grove();
    let sprig = grove.sprig();
    assert!(sprig.pollen().is_empty());

    for _ in 0..8 {
        tick(&mut grove);
    }

    assert!(sprig.pollen().is_empty());
}

/// The first reading is the value as it already stands, taken at the end of the frame the watch was
/// drained in -- so a watch does not have to be seeded by waiting for something to move.
#[test]
fn a_watch_reads_at_the_end_of_the_frame_it_was_asked_in() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(10.0)));
    sprig.watch(leaf, Vein::Drawn);

    assert_eq!(sprig.tap(leaf, Vein::Drawn), None);
    tick(&mut grove);
    assert_eq!(
        sprig.tap(leaf, Vein::Drawn),
        Some(Sap::Section(Section::from_edges(10.0, 0.0, 20.0, 10.0)))
    );
}

/// What the frame publishes is what the frame's own callsite reads, for as long as the watch stands.
#[test]
fn a_watch_follows_the_property() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));
    sprig.watch(leaf, Vein::Drawn);
    tick(&mut grove);

    grove.at(leaf, box_at(30.0));
    tick(&mut grove);

    assert_eq!(sprig.tap(leaf, Vein::Drawn), grove.tap(leaf, Vein::Drawn));
    assert_eq!(
        sprig.tap(leaf, Vein::Drawn),
        Some(Sap::Section(Section::from_edges(30.0, 0.0, 40.0, 10.0)))
    );
}

/// Watching the same property twice is one watch, and one `unwatch` ends it.
#[test]
fn unwatching_takes_the_reading_with_it() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));
    sprig.watch(leaf, Vein::Drawn);
    sprig.watch(leaf, Vein::Drawn);
    tick(&mut grove);
    assert!(sprig.tap(leaf, Vein::Drawn).is_some());

    sprig.unwatch(leaf, Vein::Drawn);
    tick(&mut grove);

    assert_eq!(sprig.tap(leaf, Vein::Drawn), None);
    assert!(grove.watched.is_empty());
}

/// A property the element does not carry reads nothing here for the reason it reads nothing at the
/// frame's own callsite.
#[test]
fn a_watch_on_what_an_element_does_not_carry_reads_nothing() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));
    sprig.watch(leaf, Vein::Color);
    tick(&mut grove);

    assert_eq!(grove.tap(leaf, Vein::Color), None);
    assert_eq!(sprig.tap(leaf, Vein::Color), None);
    // The watch stands: it ends with the element, or where `unwatch` says.
    assert!(!grove.watched.is_empty());
}

/// A name is never handed out twice, so a watch on something that has withered is one nothing can
/// answer again -- and it is dropped rather than asked every frame for the rest of the run.
#[test]
fn a_watch_ends_with_its_element() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));
    sprig.watch(leaf, Vein::Drawn);
    tick(&mut grove);
    assert!(sprig.tap(leaf, Vein::Drawn).is_some());

    sprig.prune(leaf);
    tick(&mut grove);

    assert_eq!(sprig.tap(leaf, Vein::Drawn), None);
    assert!(grove.watched.is_empty());
}

/// A watch is an op, so it is dropped where every other op naming something that has withered is
/// dropped, rather than standing as a read nothing can ever answer.
#[test]
fn a_watch_naming_a_withered_leaf_is_dropped() {
    let mut grove = grove();
    let mut sprig = grove.sprig();
    let leaf = sprig.plant(Stem::new().at(box_at(0.0)));
    tick(&mut grove);
    sprig.prune(leaf);
    tick(&mut grove);

    sprig.watch(leaf, Vein::Drawn);
    tick(&mut grove);

    assert_eq!(sprig.tap(leaf, Vein::Drawn), None);
    assert!(grove.watched.is_empty());
}

/// Nothing before the first frame, and what the frame answers for itself afterwards.
#[test]
fn conditions_are_what_the_frame_answers_frame_wide() {
    let mut grove = grove();
    let sprig = grove.sprig();
    assert_eq!(sprig.conditions(), None);

    tick(&mut grove);
    let conditions = sprig.conditions().expect("a frame has run");
    assert_eq!(conditions.viewport, grove.viewport());
    assert_eq!(conditions.layout, grove.layout());
    assert_eq!(conditions.short, grove.short());
    assert_eq!(conditions.scheme, grove.scheme());
    assert_eq!(conditions.focused, grove.focused());
    assert_eq!(conditions.frame_time, grove.frame_time());
    assert_eq!(conditions.elapsed, grove.elapsed());
}

/// Taken together from one frame, so the viewport and the breakpoint in hand always agree with each
/// other rather than being read a frame apart.
#[test]
fn conditions_are_taken_together() {
    let mut grove = grove();
    let sprig = grove.sprig();
    tick(&mut grove);
    let before = sprig.conditions().expect("a frame has run");
    assert_eq!(before.layout, crate::Layout::Xs);

    resize(&mut grove, Area::new(900.0, 700.0));
    tick(&mut grove);

    let after = sprig.conditions().expect("a frame has run");
    assert_eq!(after.viewport, Area::new(900.0, 700.0));
    assert_eq!(after.layout, crate::Layout::Lg);
}

/// Every handle is the same handle, so two workers share one stream rather than each receiving a
/// copy of it.
#[test]
fn every_handle_is_the_same_handle() {
    let mut grove = grove();
    let mut one = grove.sprig();
    let two = grove.sprig();
    assert!(one.pollen().is_empty());

    let leaf = one.plant(Stem::new().at(box_at(0.0)));
    one.watch(leaf, Vein::Drawn);
    tick(&mut grove);

    // The watch one asked for is readable through the other, and so are the conditions.
    assert_eq!(two.tap(leaf, Vein::Drawn), one.tap(leaf, Vein::Drawn));
    assert!(two.conditions().is_some());
    // And the report the second takes is one the first no longer has.
    one.prune(leaf);
    tick(&mut grove);
    tick(&mut grove);
    assert!(two.pollen().iter().any(|pollen| pollen.withered(leaf)));
    assert!(one.pollen().is_empty());
}
