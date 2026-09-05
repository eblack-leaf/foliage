//! The fonts an app registers, and the character cell each one is measured in.

use tracing::info;

use crate::coordinate::{Area, Position};
use crate::layout::{Layout, Short};
use crate::placement::breakpoints::{Breakpoints, Override};

/// The bundled font's bytes: JetBrains Mono NL Medium, under the SIL Open Font License. Its terms
/// are beside it.
const BUNDLED: &[u8] = include_bytes!("JetBrainsMonoNL-Medium.ttf");

/// The size the rasteriser optimises its glyph cache for. One number, and only a hint: every size
/// works, this one works fastest.
const OPTIMISED_FOR: f32 = 40.0;

/// Which registered font an element composes in.
///
/// Handed out by [`Foliage::font`](crate::Foliage::font) at boot and named on an element with
/// [`font`](crate::Place::font). Opaque: there is nothing to be done with one but choose it.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Font(pub(crate) u32);

impl Font {
    /// The bundled font, which is what an element that names none composes in.
    pub const DEFAULT: Self = Self(0);
}

impl Default for Font {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Glyph size in logical pixels, per breakpoint.
///
/// The same chain [`Location`](crate::Location) and [`Grid`](crate::Grid) are written in: one link
/// per breakpoint, and a breakpoint with nothing of its own takes the nearest smaller one that has.
///
/// ```no_run
/// # use foliage::FontSize;
/// FontSize::new().xs(14).lg(18).short(12);
/// ```
///
/// It is not only text's. Whatever an element is, its size is what gives
/// [`letters`](crate::Source::letters) and a letter-pitched track their pitch, so an element sized
/// in characters carries one whether or not it draws any.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FontSize(Breakpoints<Px>);

/// One breakpoint's glyph size. A newtype rather than a bare `u32` so the chain's fallback
/// terminates in a size that can be read rather than in zero.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Px(pub(crate) u32);

impl Default for Px {
    fn default() -> Self {
        Self(FontSize::DEFAULT)
    }
}

impl FontSize {
    /// The size an element that says nothing composes at, in logical pixels.
    pub const DEFAULT: u32 = 16;

    /// The default size at every breakpoint. Each of [`xs`](FontSize::xs) upward overrides one.
    pub fn new() -> Self {
        Self(Breakpoints::new())
    }

    /// States the size from the smallest breakpoint up, which is to say everywhere that a larger one
    /// does not override it.
    pub fn xs(self, size: u32) -> Self {
        self.set(Override::Xs, size)
    }

    /// Overrides the size from the `sm` breakpoint up.
    pub fn sm(self, size: u32) -> Self {
        self.set(Override::Sm, size)
    }

    /// Overrides the size from the `md` breakpoint up.
    pub fn md(self, size: u32) -> Self {
        self.set(Override::Md, size)
    }

    /// Overrides the size from the `lg` breakpoint up.
    pub fn lg(self, size: u32) -> Self {
        self.set(Override::Lg, size)
    }

    /// Overrides the size at the `xl` breakpoint.
    pub fn xl(self, size: u32) -> Self {
        self.set(Override::Xl, size)
    }

    /// Overrides the size whenever the viewport is vertically cramped, whatever its width.
    ///
    /// Width alone reads a landscape phone wrong: it is `md`-wide with a fraction of the height to
    /// put type in, and large type is what runs out of room first.
    pub fn short(self, size: u32) -> Self {
        self.set(Override::Short, size)
    }

    fn set(mut self, at: Override, size: u32) -> Self {
        self.0.set(at, Px(size));
        self
    }

    /// The size in force, in logical pixels.
    pub(crate) fn at(&self, layout: Layout, short: Short) -> u32 {
        self.0.at(layout, short).0
    }
}

impl Default for FontSize {
    fn default() -> Self {
        Self::new()
    }
}

/// Which font an element composes in, and at what size.
///
/// Placement input rather than decoration: it is the whole of what gives a character cell a size,
/// and so what [`letters`](crate::Source::letters) and a letter-pitched track resolve against. An
/// element that was never given one has no cell, and reads zero for both.
#[derive(bevy_ecs::component::Component, Copy, Clone, Debug, Default, PartialEq)]
pub(crate) struct Typeface {
    pub(crate) font: Font,
    pub(crate) size: FontSize,
}

/// Every registered font. Entry zero is the bundled one, so there is always a font to compose in.
pub(crate) struct Fonts {
    /// A slot per name. Empty where a name was handed out for a face that has not arrived yet,
    /// which is what lets a fetched font be chosen by elements before it has been read.
    faces: Vec<Option<fontdue::Font>>,
}

impl Fonts {
    pub(crate) fn new() -> Self {
        Self {
            faces: vec![Some(parse(BUNDLED))],
        }
    }

    /// Registers a font and hands back the name elements choose it by.
    ///
    /// # Panics
    ///
    /// If the font is not monospaced. See [`monospaced`].
    pub(crate) fn register(&mut self, bytes: &[u8]) -> Font {
        self.faces.push(Some(parse(bytes)));
        let font = Font(self.faces.len() as u32 - 1);
        info!(font = font.0, "font registered");
        font
    }

    /// A name for a face that has not been read yet, so elements can be composed in it now.
    ///
    /// Until it arrives the name reads as the bundled face rather than as nothing: a run has a cell
    /// either way, so a page laid out in `letters()` is laid out sensibly from the first frame and
    /// reflows when the real face lands. Measuring zero until then would have every column address
    /// on the page collapse and spring back.
    pub(crate) fn pending(&mut self) -> Font {
        self.faces.push(None);
        Font(self.faces.len() as u32 - 1)
    }

    /// Fills a name handed out by [`pending`](Fonts::pending).
    ///
    /// Refuses rather than panics: bytes that arrived from a path or a URL are not a statement the
    /// program made, so a face that turns out to be proportional is a report an app reads, where the
    /// same font written into the binary is a mistake worth stopping for.
    pub(crate) fn fill(&mut self, font: Font, bytes: &[u8]) -> Result<(), String> {
        let face = try_parse(bytes)?;
        let Some(slot) = self.faces.get_mut(font.0 as usize) else {
            return Err("no such font".to_string());
        };
        *slot = Some(face);
        info!(font = font.0, "font registered");
        Ok(())
    }

    /// The face `font` names, or the bundled one where it named nothing or has yet to arrive.
    fn face(&self, font: Font) -> &fontdue::Font {
        self.faces
            .get(font.0 as usize)
            .and_then(Option::as_ref)
            .unwrap_or_else(|| self.faces[Font::DEFAULT.0 as usize].as_ref().expect("bundled"))
    }

    /// One character cell of `font` at `size`: the advance every glyph shares, and the distance
    /// between two baselines.
    ///
    /// Both are taken up to whole logical pixels. A cell is the pitch a whole run is addressed on --
    /// by [`letters`](crate::Source::letters), by a letter-pitched track, and by wrapping -- so a
    /// fractional one would accumulate along a line and put the hundredth column somewhere other
    /// than a hundred cells in.
    pub(crate) fn cell(&self, font: Font, size: u32) -> Area {
        let face = self.face(font);
        let px = size as f32;
        let advance = face.metrics(REFERENCE, px).advance_width;
        let line = face
            .horizontal_line_metrics(px)
            .map(|metrics| metrics.new_line_size)
            .unwrap_or(px);
        Area::new(advance.ceil(), line.ceil())
    }

    /// Rasterises one character into coverage, and says where inside its cell the ink belongs.
    ///
    /// `scale` is the display's device pixels per logical one. The bitmap is made at that density --
    /// a glyph cut at logical size and magnified is a blurred glyph -- while everything said about
    /// where it goes stays in logical pixels, because that is the one unit the engine is written in.
    ///
    /// The offset is the whole of the typesetting: a cell is a box on a grid, and the ink sits
    /// inside it at the face's own left bearing, on the face's own baseline. Nothing above this
    /// line knows a glyph has a bearing at all.
    ///
    /// # Three samples, one channel
    ///
    /// Taken through the rasteriser's subpixel path and averaged back into a single coverage byte.
    /// That is **horizontal supersampling and not subpixel antialiasing**: keeping the three samples
    /// as red, green and blue would need per-channel alpha to composite -- dual-source blending,
    /// which WebGL2 does not have -- and would assume a physical stripe order that is wrong on a
    /// rotated or pentile display. Averaged, they are three samples per pixel instead of one, which
    /// costs no format, no shader and no feature test.
    ///
    /// What it buys is the case single-sample coverage estimates worst: a vertical stem narrower
    /// than a pixel, or one straddling two. That is small type, and it is round letters, whose bowls
    /// are two thin vertical stems.
    pub(crate) fn rasterize(&self, font: Font, size: u32, scale: f32, character: char) -> Ink {
        let face = self.face(font);
        let px = size as f32 * scale;
        let (metrics, coverage) = face.rasterize_subpixel(character, px);
        // The metrics are the same ones the single-sample path reports, so no glyph moves for
        // having been sampled three times.
        let coverage = match metrics.width == 0 || metrics.height == 0 {
            true => coverage,
            false => coverage
                .chunks_exact(3)
                .map(|pixel| {
                    ((pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16 + 1) / 3) as u8
                })
                .collect(),
        };
        // Where the baseline sits below the top of the cell, in logical pixels. The cell is the
        // face's own line height, so its baseline is the face's own ascent.
        let baseline = face
            .horizontal_line_metrics(size as f32)
            .map(|metrics| metrics.ascent)
            .unwrap_or(size as f32);
        // `ymin` is the bitmap's bottom edge measured up from the baseline, so its top edge is that
        // plus its height -- and the offset down from the top of the cell is what is left.
        let above = (metrics.ymin + metrics.height as i32) as f32 / scale;
        Ink {
            coverage,
            width: metrics.width as u32,
            height: metrics.height as u32,
            offset: Position::new(metrics.xmin as f32 / scale, baseline - above),
        }
    }
}

/// The ink of one character, as the rasteriser made it.
///
/// Two units, and they are not interchangeable: the bitmap is device pixels, because that is what a
/// texture holds, and everything about where it goes is logical, because that is what a box is
/// measured in.
pub(crate) struct Ink {
    /// Coverage, one byte per device pixel, row-major and `width` wide.
    pub(crate) coverage: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Where the ink's top-left corner sits inside its character cell, in logical pixels.
    pub(crate) offset: Position,
}

/// The character every cell is measured from. Any of them would do in a monospaced font, which is
/// what [`monospaced`] is checked for.
const REFERENCE: char = 'a';

/// Characters spanning the narrowest and widest shapes a Latin font has. If these agree on advance
/// width, it is monospaced.
const PROBE: [char; 6] = ['i', 'W', 'm', '.', '1', 'g'];

/// How far apart two advances may be and still be called the same.
const TOLERANCE: f32 = 0.01;

fn parse(bytes: &[u8]) -> fontdue::Font {
    match try_parse(bytes) {
        Ok(font) => font,
        Err(refused) => panic!("{refused}"),
    }
}

/// The same reading, for bytes the program did not write into itself.
///
/// One function so the two roads cannot come to disagree about what a usable font is: what a fetched
/// face is held to is exactly what a bundled one is, and only what happens to a refusal differs.
fn try_parse(bytes: &[u8]) -> Result<fontdue::Font, String> {
    let font = fontdue::Font::from_bytes(
        bytes,
        fontdue::FontSettings {
            scale: OPTIMISED_FOR,
            ..fontdue::FontSettings::default()
        },
    )
    .map_err(|failed| format!("the font could not be parsed: {failed}"))?;
    let px = OPTIMISED_FOR;
    let advances = PROBE.into_iter().filter_map(|character| {
        // A glyph index of zero is `.notdef`: the font has no such character, and measuring the
        // fallback would compare it against itself.
        match font.lookup_glyph_index(character) {
            0 => None,
            _ => Some((character, font.metrics(character, px).advance_width)),
        }
    });
    if let Err((first, one, second, other)) = monospaced(advances) {
        return Err(format!(
            "font is not monospaced: '{first}' advances {one}px but '{second}' advances {other}px \
             at {px}px. foliage sizes, divides and wraps by a fixed character cell -- `letters()`, \
             a letter-pitched track, and every measured extent -- so a proportional font \
             mispositions rather than merely looking different."
        ));
    }
    Ok(font)
}

/// Whether every advance given is the same one, and which pair disagreed if not.
///
/// Checked at registration rather than left to the renderer, because the whole placement model is
/// built on a fixed advance: a proportional font does not degrade, it silently puts every column
/// address somewhere it does not belong. A refusal where the font is handed over names the problem
/// while it can still be fixed.
pub(crate) fn monospaced(
    advances: impl IntoIterator<Item = (char, f32)>,
) -> Result<(), (char, f32, char, f32)> {
    let mut reference: Option<(char, f32)> = None;
    for (character, advance) in advances {
        match reference {
            None => reference = Some((character, advance)),
            Some((first, one)) if (advance - one).abs() > TOLERANCE => {
                return Err((first, one, character, advance));
            }
            Some(_) => {}
        }
    }
    Ok(())
}
