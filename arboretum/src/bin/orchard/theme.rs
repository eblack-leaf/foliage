//! What the shop is printed in: label stock on a paper ground, one ink, and one oxblood.
//!
//! The reason for all of it is the label. Every good is weighed, printed and sealed, so the shop
//! is set the way a label is: flat ground, no shading, a near-black ink for everything that is
//! said, heavy and hairline rules to divide it, and the oxblood of sealing wax kept for the one
//! press to make and the marks the till leaves. The colour on the page is the goods' own.

use foliage::{
    Color, Font, FontSize, Horizontal, Location, Palette, Scheme, Source, Vertical, center_x,
};

use crate::catalog::Lch;

/// The ground: uncoated paper.
pub const PAPER: Lch = (0.952, 0.011, 84.0);
/// What a label is printed on.
pub const STOCK: Lch = (0.992, 0.004, 90.0);
/// The grey between.
pub const GREY: Lch = (0.60, 0.012, 70.0);
/// The ink.
pub const INK: Lch = (0.205, 0.012, 45.0);
/// Sealing wax.
pub const OXBLOOD: Lch = (0.42, 0.13, 20.0);
/// The wax, pale: a choice the person made, filled.
const ROSE: Lch = (0.89, 0.035, 25.0);

/// The wax, as a seal is grown from it: deep to light.
const WAX: [Lch; 4] = [
    (0.30, 0.10, 16.0),
    (0.39, 0.13, 20.0),
    (0.49, 0.15, 24.0),
    (0.60, 0.13, 30.0),
];

/// Which spectrum the seal is grown along.
pub const SEAL: usize = 0;

/// How wide the page's column is at most.
pub const COLUMN: f32 = 1240.0;

/// How wide a heavy rule is, and a hairline.
pub const HEAVY: f32 = 3.0;
pub const HAIR: f32 = 1.0;

pub fn oklch((l, c, h): Lch) -> Color {
    Color::oklch(l, c, h)
}

/// The wax a seal is pressed from, deep to light.
pub fn wax() -> Vec<Color> {
    WAX.map(oklch).to_vec()
}

/// The shop's reading.
pub fn scheme() -> Scheme {
    Scheme::light()
        .set(Palette::Surface, oklch(PAPER))
        .set(Palette::Raised, oklch(STOCK))
        .set(Palette::Muted, oklch(GREY))
        .set(Palette::Ink, oklch(INK))
        .set(Palette::Accent, oklch(OXBLOOD))
        .set(Palette::Accent.mark(), oklch(OXBLOOD))
        .set(Palette::Signal, oklch(ROSE))
        .set(Palette::Signal.mark(), oklch(OXBLOOD))
        .set(Palette::Positive, oklch(INK))
        .set(Palette::Danger, oklch((0.52, 0.19, 27.0)))
        .set(Palette::Caution, oklch((0.76, 0.14, 75.0)))
        .set(Palette::Caution.mark(), oklch((0.50, 0.11, 58.0)))
        .spectrum(SEAL, &WAX.map(oklch))
}

/// The faces the shop is set in: Xanh Mono, upright and italic, for its voice, and the specimen
/// posters' own.
#[derive(Copy, Clone, Debug)]
pub struct Faces {
    pub roman: Font,
    pub italic: Font,
    pub major: Font,
    pub plex: Font,
}

/// The page's column, `top` as `vertical` says: the window less its gutters, no wider than
/// [`COLUMN`].
pub fn column(vertical: impl Fn() -> Vertical) -> Location {
    Location::new()
        .xs(across(16.0), vertical())
        .md(across(28.0), vertical())
        .lg(across(36.0), vertical())
}

fn across(gutter: f32) -> Horizontal {
    center_x(50.pct())
        .width(100.pct() - (2.0 * gutter).px())
        .at_most(COLUMN.px())
}

/// How tall the bar over the shop stands: two rows on a narrow window, one on a wider.
pub const BAR_NARROW: f32 = 104.0;
pub const BAR_WIDE: f32 = 64.0;

/// The headline: two lines however wide the window, so its size is what fits seventeen of its
/// half-em letters across.
pub fn headline() -> FontSize {
    FontSize::new().xs(40).sm(46).md(64).lg(88).xl(104)
}

/// A section's heading.
pub fn heading() -> FontSize {
    FontSize::new().xs(28).md(36)
}

/// A good's name, large: the detail's title.
pub fn title() -> FontSize {
    FontSize::new().xs(32).md(44)
}

/// A good's name on its label in the grid.
pub fn name() -> FontSize {
    FontSize::new().xs(16).md(18).lg(19)
}

/// A price on a label.
pub fn price() -> FontSize {
    FontSize::new().xs(18).md(20)
}

/// A figure the till or the scale reads out.
pub fn figure() -> FontSize {
    FontSize::new().xs(30).md(38)
}

pub fn lead() -> FontSize {
    FontSize::new().xs(15).md(17)
}

pub fn body() -> FontSize {
    FontSize::new().xs(14).md(15)
}

/// The small print: what a label's rows are set in.
pub fn small() -> FontSize {
    FontSize::new().xs(12)
}

#[cfg(test)]
mod tests {
    use crate::catalog::GOODS;

    const ROMAN: &[u8] = include_bytes!("../../../assets/fonts/xanh-mono/XanhMono-Regular.ttf");
    const ITALIC: &[u8] = include_bytes!("../../../assets/fonts/xanh-mono/XanhMono-Italic.ttf");

    fn u16_at(data: &[u8], at: usize) -> u32 {
        u32::from(u16::from_be_bytes([data[at], data[at + 1]]))
    }

    fn u32_at(data: &[u8], at: usize) -> u32 {
        u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
    }

    /// The glyph `c` maps to in `font`'s Unicode cmap: zero where it has none.
    fn glyph(font: &[u8], c: char) -> u32 {
        let tables = u16_at(font, 4) as usize;
        let cmap = (0..tables)
            .map(|n| 12 + 16 * n)
            .find(|&record| &font[record..record + 4] == b"cmap")
            .map(|record| u32_at(font, record + 8) as usize)
            .expect("a cmap");
        let code = c as u32;
        let encodings = u16_at(font, cmap + 2) as usize;
        for n in 0..encodings {
            let record = cmap + 4 + 8 * n;
            let (platform, encoding) = (u16_at(font, record), u16_at(font, record + 2));
            if !matches!((platform, encoding), (0, _) | (3, 1) | (3, 10)) {
                continue;
            }
            let sub = cmap + u32_at(font, record + 4) as usize;
            match u16_at(font, sub) {
                4 => {
                    let segments = u16_at(font, sub + 6) as usize / 2;
                    let ends = sub + 14;
                    let starts = ends + 2 * segments + 2;
                    let deltas = starts + 2 * segments;
                    let ranges = deltas + 2 * segments;
                    for s in 0..segments {
                        let (start, end) =
                            (u16_at(font, starts + 2 * s), u16_at(font, ends + 2 * s));
                        if code < start || code > end {
                            continue;
                        }
                        let delta = u16_at(font, deltas + 2 * s);
                        let range = u16_at(font, ranges + 2 * s) as usize;
                        let id = match range {
                            0 => code,
                            _ => {
                                let at = ranges + 2 * s + range + 2 * (code - start) as usize;
                                match u16_at(font, at) {
                                    0 => return 0,
                                    id => id,
                                }
                            }
                        };
                        return (id + delta) & 0xffff;
                    }
                }
                12 => {
                    let groups = u32_at(font, sub + 12) as usize;
                    for g in 0..groups {
                        let at = sub + 16 + 12 * g;
                        let (start, end) = (u32_at(font, at), u32_at(font, at + 4));
                        if (start..=end).contains(&code) {
                            return u32_at(font, at + 8) + code - start;
                        }
                    }
                }
                _ => {}
            }
        }
        0
    }

    #[test]
    fn xanh_mono_has_every_glyph_the_shop_sets_in_it() {
        let mut set: String = GOODS.iter().map(|g| g.name).collect();
        set.push_str("bits & bytes Digital goods, sold by the byte. €0123456789. −·×“”…");
        set.push_str("The aisles Your bag The till Receipt Sealed in wax Colophon PAID");
        for font in [ROMAN, ITALIC] {
            let missing: String = set
                .chars()
                .filter(|&c| c != ' ' && glyph(font, c) == 0)
                .collect();
            assert!(missing.is_empty(), "missing {missing:?}");
        }
        assert_eq!(
            glyph(ROMAN, '\u{e000}'),
            0,
            "a private-use point has no glyph"
        );
    }
}
