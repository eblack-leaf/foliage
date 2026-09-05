//! Bytes that arrive from outside a frame.
//!
//! What reads them -- a thread, a promise -- is the platform edge the suite cannot cover, exactly as
//! the winit translation is. What it *produces* is an op, so a test pushes that op where a finished
//! retrieval pushes it and everything past that point is one path.

use std::io::Cursor;

use crate::asset::{Destination, Retrieved};
use crate::coordinate::Area;
use crate::icon::Field;
use crate::image::Plate;
use crate::op::Op;
use crate::tests::{Observer, grove, section, tick, tick_with};
use crate::{
    Boxed, Fit, Font, FontSize, Grove, Grow, Icon, Image, Location, Origin, Place, Pollen, Section,
    Source, Text, content, left, top,
};

/// A box at a stated place.
fn at(width: f32, height: f32) -> Location {
    Location::new().xs(
        left(0.px()).width(width.px()),
        top(0.px()).height(height.px()),
    )
}

/// A PNG of a stated size, encoded here rather than kept as a fixture: what matters is that the
/// bytes are a real picture the decoder answers, not what is in it.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
        width,
        height,
        image::Rgba([10, 20, 30, 255]),
    ))
    .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
    .expect("a png");
    bytes
}

/// A retrieval finishing, entering the pipeline where a thread or a promise enters it.
fn arrives(grove: &mut Grove, destination: Destination, bytes: Retrieved) {
    grove.queue.push(Op::Arrived { destination, bytes });
}

/// Runs one frame with an app in it and hands back what that frame told it.
fn frame(grove: &mut Grove) -> Pollen {
    let mut app = Observer::default();
    tick_with(grove, &mut app);
    app.last().clone()
}

/// Bytes an app holds are registered in the call that was handed them, and a picture's size is what
/// the decode says rather than what anyone stated about it.
#[test]
fn a_picture_given_outright_is_decoded_at_once() {
    let mut grove = grove();
    let plate = grove.image(png(3, 2));
    assert_eq!(grove.plates.size(plate), Some(Area::new(3.0, 2.0)));
}

/// The bundled case has to stay a one-liner, which means `include_bytes!` has to be passable as it
/// is -- and what it produces is a reference to an array rather than a slice.
#[test]
fn bundled_bytes_are_passed_as_they_come() {
    let mut grove = grove();
    let font = grove.font(include_bytes!("../text/JetBrainsMonoNL-Medium.ttf"));
    assert_ne!(font, Font::DEFAULT);
}

/// A name is valid from the moment it is handed out. An element drawing a picture that has not
/// arrived occupies its box and draws nothing, and appears in the frame its pixels do -- with
/// nothing to undo, because it was never in the batch.
#[test]
fn a_name_is_valid_before_its_bytes_are() {
    let mut grove = grove();
    let plate = grove.plates.name();
    let leaf = grove.plant(Image::new(plate).fit(Fit::Crop).at(at(40.0, 40.0)));
    tick(&mut grove);
    assert_eq!(grove.elm.images.len(), 0);
    assert_eq!(section(&grove, leaf), Section::from_edges(0.0, 0.0, 40.0, 40.0));

    arrives(&mut grove, Destination::Picture(plate), Ok(png(8, 8)));
    tick(&mut grove);
    assert_eq!(grove.elm.images.len(), 1);
    // Drained at step 4, and handed to the app at step 3 of the next frame like every other report.
    assert!(frame(&mut grove).loaded(plate));
}

/// One question over three handles, because one road fills all three.
#[test]
fn a_font_a_mark_and_a_picture_are_asked_the_same_way() {
    let mut grove = grove();
    let (font, field, plate) = (
        grove.fonts.pending(),
        grove.fields.pending(),
        grove.plates.name(),
    );
    arrives(
        &mut grove,
        Destination::Face(font),
        Ok(include_bytes!("../text/JetBrainsMonoNL-Medium.ttf").to_vec()),
    );
    arrives(
        &mut grove,
        Destination::Mark(field, 4, 2.0),
        Ok(vec![255; 4 * 4 * 4]),
    );
    arrives(&mut grove, Destination::Picture(plate), Ok(png(2, 2)));
    tick(&mut grove);

    let heard = frame(&mut grove);
    assert!(heard.loaded(font));
    assert!(heard.loaded(field));
    assert!(heard.loaded(plate));
}

/// Bytes that never arrived and bytes that turned out to be something else are one report, because
/// there is one thing an app can do about either.
#[test]
fn what_could_not_be_read_or_used_is_missing() {
    let mut grove = grove();
    let (unread, unparsable, undecodable) = (
        grove.plates.name(),
        grove.fonts.pending(),
        grove.plates.name(),
    );
    arrives(
        &mut grove,
        Destination::Picture(unread),
        Err("no such file".to_string()),
    );
    // A font that will not parse is refused rather than panicked on: what a path or a URL turned out
    // to hold is not something the program stated.
    arrives(&mut grove, Destination::Face(unparsable), Ok(vec![0, 1, 2]));
    arrives(
        &mut grove,
        Destination::Picture(undecodable),
        Ok(b"not a picture".to_vec()),
    );
    tick(&mut grove);

    let heard = frame(&mut grove);
    assert!(heard.missing(unread));
    assert!(heard.missing(unparsable));
    assert!(heard.missing(undecodable));
    assert!(!heard.loaded(unread));
    // The name stays valid and unfilled: what drew nothing goes on drawing nothing.
    assert_eq!(grove.plates.size(unread), None);
}

/// A URL is nameable on every target and fetched on the web. Off it the name is taken and answered
/// as missing, so a program that states its assets' URLs states them once and what is outstanding is
/// an implementation rather than a surface.
#[test]
fn a_url_is_missing_where_it_cannot_be_fetched() {
    let mut grove = grove();
    let plate = grove.image(Origin::url("https://example.invalid/logo.png"));
    tick(&mut grove);
    assert!(frame(&mut grove).missing(plate));
    assert_eq!(grove.plates.size(plate), None);
}

/// A run composed in a face that has not landed is measured in the bundled one, so a page laid out
/// in `letters()` is laid out sensibly from the first frame. Measuring zero until then would have
/// every column address on the page collapse and spring back.
#[test]
fn a_face_that_has_not_arrived_is_measured_as_the_bundled_one() {
    let mut grove = grove();
    let pending = grove.fonts.pending();
    let measured = |grove: &mut Grove, font: Font| {
        let run = grove.plant(
            Text::new("hello")
                .font(font)
                .font_size(FontSize::new().xs(16))
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(content()),
                )),
        );
        tick(grove);
        section(grove, run)
    };
    assert_eq!(measured(&mut grove, pending), measured(&mut grove, Font::DEFAULT));
}

/// A mark that has not arrived is the picture's case, and answers the same way: the element is
/// placed, ranked and in the stack, and draws nothing until the field lands.
#[test]
fn a_mark_that_has_not_arrived_draws_nothing_until_it_does() {
    let mut grove = grove();
    let field: Field = grove.fields.pending();
    let leaf = grove.plant(Icon::new(field).at(at(24.0, 24.0)));
    tick(&mut grove);
    assert_eq!(grove.elm.icons.len(), 0);
    assert_eq!(section(&grove, leaf), Section::from_edges(0.0, 0.0, 24.0, 24.0));

    arrives(
        &mut grove,
        Destination::Mark(field, 4, 2.0),
        Ok(vec![255; 4 * 4 * 4]),
    );
    tick(&mut grove);
    assert_eq!(grove.elm.icons.len(), 1);
}

/// A field that arrived too small for what it was said to be is refused, where the same bytes given
/// outright would have been an assertion. The two roads hold a field to the same thing and differ
/// only in what a refusal does.
#[test]
fn a_field_smaller_than_it_was_said_to_be_is_missing() {
    let mut grove = grove();
    let field = grove.fields.pending();
    arrives(&mut grove, Destination::Mark(field, 8, 2.0), Ok(vec![255; 16]));
    tick(&mut grove);
    assert!(frame(&mut grove).missing(field));
    let leaf = grove.plant(Icon::new(field).at(at(24.0, 24.0)));
    tick(&mut grove);
    assert_eq!(grove.elm.icons.len(), 0);
    assert_eq!(section(&grove, leaf), Section::from_edges(0.0, 0.0, 24.0, 24.0));
}

/// A plate whose pixels are replaced reaches every element drawing it without any of them being
/// named -- which is what makes a re-fetch at a higher resolution one write.
#[test]
fn a_picture_that_arrives_twice_keeps_its_name() {
    let mut grove = grove();
    let plate: Plate = grove.plates.name();
    grove.plant(Image::new(plate).at(at(40.0, 40.0)));
    arrives(&mut grove, Destination::Picture(plate), Ok(png(2, 2)));
    tick(&mut grove);
    assert_eq!(grove.plates.size(plate), Some(Area::new(2.0, 2.0)));

    arrives(&mut grove, Destination::Picture(plate), Ok(png(16, 16)));
    tick(&mut grove);
    assert_eq!(grove.plates.size(plate), Some(Area::new(16.0, 16.0)));
    assert_eq!(grove.elm.images.len(), 1);
}
