//! The fonts an app registers, and the character cell each one is measured in.

use tracing::info;

use crate::coordinate::Area;
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
    faces: Vec<fontdue::Font>,
}

impl Fonts {
    pub(crate) fn new() -> Self {
        Self {
            faces: vec![parse(BUNDLED)],
        }
    }

    /// Registers a font and hands back the name elements choose it by.
    ///
    /// # Panics
    ///
    /// If the font is not monospaced. See [`monospaced`].
    pub(crate) fn register(&mut self, bytes: &[u8]) -> Font {
        self.faces.push(parse(bytes));
        let font = Font(self.faces.len() as u32 - 1);
        info!(font = font.0, "font registered");
        font
    }

    /// The face `font` names, or the bundled one if it was never registered.
    fn face(&self, font: Font) -> &fontdue::Font {
        self.faces
            .get(font.0 as usize)
            .unwrap_or(&self.faces[Font::DEFAULT.0 as usize])
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
    let font = fontdue::Font::from_bytes(
        bytes,
        fontdue::FontSettings {
            scale: OPTIMISED_FOR,
            ..fontdue::FontSettings::default()
        },
    )
    .expect("a parsable font");
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
        panic!(
            "font is not monospaced: '{first}' advances {one}px but '{second}' advances {other}px \
             at {px}px. foliage sizes, divides and wraps by a fixed character cell -- `letters()`, \
             a letter-pitched track, and every measured extent -- so a proportional font \
             mispositions rather than merely looking different."
        );
    }
    font
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
