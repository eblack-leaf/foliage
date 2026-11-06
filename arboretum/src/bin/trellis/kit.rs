//! The drafting kit every part of the sheet is drawn with: an instrument's plate, lettering, and
//! rules.

use foliage::{
    Boxed, Cap, Color, Elevation, Fill, Font, Grove, Grow, Leaf, Line, Location, Palette, Panel,
    Place, Point, Source, Text, content, left, top,
};

use crate::theme::{self, Faces, size};

/// How far outside a plate's corners its registration ticks stand, and how long each arm is.
const TICK_OFF: f32 = 4.0;
const TICK_ARM: f32 = 9.0;

/// An instrument's plate: a graphite ground ruled round in hairline, with a registration tick at
/// each corner in graphite on the paper, which takes every press that lands on it -- so nothing pressed on an instrument reaches
/// the paper under it, and a stray tap leaves focus on the plate rather than nowhere.
pub fn plate(grove: &mut Grove, under: Leaf, lift: Elevation) -> Leaf {
    let plate = grove.branch(
        under,
        Panel::new()
            .color(Palette::Surface)
            .outline(1.0, theme::rule())
            .interactive()
            .elevate(lift)
            .font_size(lichen::caption())
            .at(Location::new()),
    );
    ticks(grove, plate, theme::line(), 1.0);
    plate
}

/// Registration ticks round `on`'s corners, just outside them.
pub fn ticks(grove: &mut Grove, on: Leaf, fill: Color, weight: f32) -> Vec<Leaf> {
    let (o, a) = (TICK_OFF, TICK_ARM);
    let mut leaves = Vec::with_capacity(8);
    for (x, sx) in [(0.0, -1.0), (100.0, 1.0)] {
        for (y, sy) in [(0.0, -1.0), (100.0, 1.0)] {
            let corner = (x.pct() + (sx * o).px(), y.pct() + (sy * o).px());
            for (dx, dy) in [(-sx * a, 0.0), (0.0, -sy * a)] {
                leaves.push(grove.branch(
                    on,
                    Line::new()
                        .between(
                            Point::new(corner.0.clone(), corner.1.clone()),
                            Point::new(corner.0.clone() + dx.px(), corner.1.clone() + dy.px()),
                        )
                        .weight(weight)
                        .color(fill)
                        .cap(Cap::Butt)
                        .intangible()
                        .elevate(Elevation::up(1)),
                ));
            }
        }
    }
    leaves
}

/// A run of the drawing's lettering, in `font` at `px`, read in `fill`, standing where `at` says.
pub fn letter(
    grove: &mut Grove,
    under: Leaf,
    text: impl Into<String>,
    font: Font,
    px: u32,
    fill: impl Into<Fill>,
    at: Location,
) -> Leaf {
    grove.branch(
        under,
        Text::new(text)
            .font(font)
            .font_size(size(px))
            .color(fill)
            .intangible()
            .elevate(Elevation::up(1))
            .at(at),
    )
}

/// An instrument's heading: small capitals, quiet, at its top left.
pub fn heading(grove: &mut Grove, under: Leaf, faces: &Faces, text: &str, x: f32, y: f32) -> Leaf {
    letter(
        grove,
        under,
        text,
        faces.bold,
        theme::HEAD,
        theme::soft(),
        Location::new().xs(left(x.px()).width(content()), top(y.px()).height(1.letters())),
    )
}

/// A hairline rule, a panel a pixel thick, standing where `at` says.
pub fn rule(grove: &mut Grove, under: Leaf, at: Location, fill: impl Into<Fill>) -> Leaf {
    grove.branch(under, Panel::new().color(fill).intangible().elevate(Elevation::up(1)).at(at))
}

/// A ring: a panel filled with nothing, outlined, and rounded all the way.
pub fn ring(weight: f32, fill: impl Into<Fill>) -> Panel {
    Panel::new()
        .color(Color::rgba(0.0, 0.0, 0.0, 0.0))
        .outline(weight, fill)
        .rounding(foliage::Corners::all(foliage::Rounding::Full))
}
