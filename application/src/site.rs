//! The page: the leaf, and the label beside it.
//!
//! The leaf is mounted the way a herbarium sheet mounts a specimen, and the label is what a sheet's
//! corner carries: what it is, and where the rest of the collection is kept -- the three apps
//! grown on foliage, and its source. Nothing on the leaf is pressed. Pointing at a way off the page,
//! or giving it focus, lights the patch of the leaf it stands for in green, and its tile in the
//! label turns with it.
//!
//! On a window wide enough the two stand side by side, centred as a pair; on a narrow one the leaf
//! stands over the label, as large as leaves the label in view. Both are placements of the same
//! things, stated again whenever the window is resized, and the page scrolls only where a window is
//! too short for either.

use foliage::{
    Area, Axes, Boxed, Color, Corners, Ease, Elevation, FontSize, Grove, Grow, HAIRLINE, Icon, Layout,
    Leaf, Location, Motion, Palette, Panel, Place, Polygon, Pollen, Position, Root, Rounding,
    Short, Source, Stem, Text, Timing, anchor, center_y, content, left, right, top,
};
use lichen::{Hover, Mosaic, Ramp, Scatter};
use tracing::info;

use crate::icons::Icons;
use crate::leaf;
use crate::theme::{self, MARGIN};

/// One way off the page.
struct Way {
    name: &'static str,
    /// What is there, said under its name.
    says: &'static str,
    /// Where it is, from the site's own root on the web -- which holds whether or not the address
    /// the site was read from ends in a slash -- or outright.
    at: Where,
}

enum Where {
    /// A page of the site's own, by its path from the site's root.
    Here(&'static str),
    /// Somewhere else altogether.
    Away(&'static str),
}

/// Where the site is published.
const PUBLISHED: &str = "https://eblack-leaf.github.io";

const WAYS: [Way; 4] = [
    Way {
        name: "conservatory",
        says: "an exhibition of grown works",
        at: Where::Here("/foliage/portfolio/conservatory/"),
    },
    Way {
        name: "orchard",
        says: "a boutique of digital goods",
        at: Where::Here("/foliage/portfolio/orchard/"),
    },
    Way {
        name: "trellis",
        says: "a planning sheet for a team",
        at: Where::Here("/foliage/portfolio/trellis/"),
    },
    Way {
        name: "github",
        says: "eblack-leaf/foliage",
        at: Where::Away("https://github.com/eblack-leaf/foliage"),
    },
];

impl Way {
    /// Where pressing it goes: on the web, a page of the site by its path; anywhere else, the
    /// published page.
    fn href(&self) -> String {
        match self.at {
            Where::Here(path) if cfg!(target_family = "wasm") => path.to_string(),
            Where::Here(path) => format!("{PUBLISHED}{path}"),
            Where::Away(url) => url.to_string(),
        }
    }
}

/// What the label says under the wordmark.
const TAGLINE: &str = "a ui library for rust, and three apps grown on it";

/// The wordmark's size at `xs`, `md` and `lg`, and a way's name's at `xs` and `md`. A window too
/// short for either takes the `xs` size whatever its width.
const WORDMARK: [u32; 3] = [34, 44, 52];
const NAME: [u32; 2] = [17, 19];

/// How tall a line is, and how wide a letter, per pixel of its font size: the bundled face's own.
const CELL: f32 = 1.32;
const ADVANCE: f32 = 0.6;

/// What quiet words are read in: what is there, under a way's name, and the tagline. Ink, a step
/// into the ground, rather than the muted role, which is a hint's colour and too faint to read
/// whole lines in.
const QUIET: Palette = Palette::Ink.recede();

/// How far a way's ground reaches past its words, above and below, and past the label's edges.
const PAD: f32 = 12.0;

/// How far in from the label's edge a way's words start, past its tile.
const INDENT: f32 = 28.0;

/// How large a way's tile is, and the arrow at its end.
const TILE: f32 = 13.0;
const ARROW: f32 = 16.0;

/// The room between one way's ground and the next, and before the first.
const SPACE: f32 = 6.0;
const LEAD: f32 = 26.0;

/// How long a way takes to light, or to go out.
const LIGHT_MS: u64 = 220;

/// How wide the label is, at least and at most.
const COLUMN: (f32, f32) = (300.0, 420.0);

/// The narrowest window the two stand side by side on, and how much wider than tall it has to be.
const BESIDE: f32 = 720.0;
const LANDSCAPE: f32 = 0.9;

/// How much of a window's height the leaf takes, beside the label.
const SHARE: f32 = 0.94;

/// How wide the leaf is at most, and at least, over the label.
const STACKED: (f32, f32) = (440.0, 160.0);

/// The room between the leaf and the label, over it.
const UNDER: f32 = 28.0;

/// The app.
pub(crate) struct Site {
    mosaic: Mosaic,
    /// What the leaf is grown in, placed again on every resize.
    leaf: Leaf,
    /// What the label is grown in, placed again on every resize.
    label: Leaf,
    aspect: f32,
    ways: Vec<Grown>,
    hover: Hover,
    /// The ramp a lit patch is recoloured onto.
    patch: Ramp,
}

/// One way, grown.
struct Grown {
    /// The one element of it that takes a press: the ground under its words.
    ground: Leaf,
    tile: Leaf,
    arrow: Leaf,
    hue: Color,
    lit: bool,
}

impl Root for Site {
    fn take_root(grove: &mut Grove) -> Self {
        let scheme = theme::scheme();
        grove.repaint(scheme);
        let icons = grove.marks::<Icons>();
        let italic = theme::italic(grove);
        let window = grove.plant(Panel::new().color(Palette::Surface));
        let page = grove.branch(
            window,
            Stem::new()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .scrolls(Axes::Vertical),
        );
        let mut mosaic = leaf::mosaic(Ramp::of(&scheme.stops(theme::SPECIMEN)));
        let aspect = mosaic.aspect();
        let arranged = Arranged::of(aspect, grove.viewport());
        let shape = mosaic.grow(grove, page, arranged.leaf, &mut Scatter::new());
        mosaic.regions(grove, &leaf::stood());
        mosaic.form(grove);

        let label = grove.branch(
            page,
            Stem::new()
                .intangible()
                .elevate(Elevation::up(2))
                .at(arranged.label),
        );
        let wordmark = grove.branch(
            label,
            Text::new("foliage")
                .color(Palette::Ink)
                .font_size(
                    FontSize::new()
                        .xs(WORDMARK[0])
                        .md(WORDMARK[1])
                        .lg(WORDMARK[2])
                        .short(WORDMARK[0]),
                )
                .intangible()
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(0.px()).height(content()),
                )),
        );
        let tagline = grove.branch(
            label,
            Text::new(TAGLINE)
                .color(QUIET)
                .font(italic)
                .font_size(lichen::caption())
                .intangible()
                .anchored(wordmark)
                .at(Location::new().xs(
                    left(0.px()).width(100.pct()),
                    top(anchor().bottom() + 4.px()).height(content()),
                )),
        );
        let mut arrivals = vec![vec![wordmark], vec![tagline]];
        let mut above = tagline;
        let mut ways = Vec::new();
        for (n, (way, hue)) in WAYS.iter().zip(theme::HUES).enumerate() {
            let (grown, parts) = grow_way(grove, label, above, n, way, hue, icons.arrow_right);
            above = grown.ground;
            ways.push(grown);
            arrivals.push(parts);
        }
        // Room under the last way, so a page short enough to scroll scrolls to a margin past it.
        grove.branch(
            label,
            Stem::new().intangible().anchored(above).at(Location::new().xs(
                left(0.px()).width(1.px()),
                top(anchor().bottom()).height(MARGIN.px()),
            )),
        );
        // The label comes in a line at a time while the leaf is still forming, so the ways off
        // the page are there to press without waiting on it.
        for (n, parts) in arrivals.iter().enumerate() {
            let timing = Timing::ms(480)
                .after(240 + 90 * n as u64)
                .ease(Ease::Decelerate);
            for &part in parts {
                grove.opacity(part, 0.0);
                grove.shift(part, Position::new(0.0, 8.0));
                grove.animate(part, Motion::Opacity(1.0), timing);
                grove.animate(part, Motion::Shift(Position::default()), timing);
            }
        }
        let mut hover = Hover::new(grove);
        for way in &ways {
            hover.watch(way.ground);
        }
        info!("leaf planted");
        Site {
            mosaic,
            leaf: shape,
            label,
            aspect,
            ways,
            hover,
            patch: Ramp::of(&scheme.stops(theme::PATCH)),
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        if let Some(viewport) = pollen.resized() {
            let arranged = Arranged::of(self.aspect, viewport);
            grove.at(self.leaf, arranged.leaf);
            grove.at(self.label, arranged.label);
        }
        self.mosaic.frame(grove, &pollen);
        self.hover.frame(grove, true);
        let focused = grove.focused();
        for (n, way) in self.ways.iter_mut().enumerate() {
            let lit = self.hover.is_over(way.ground) || focused == Some(way.ground);
            if lit != way.lit {
                way.lit = lit;
                way.light(grove, lit);
                match lit {
                    true => self.mosaic.emphasize(grove, n, &self.patch),
                    false => self.mosaic.unemphasize(grove, n),
                }
            }
            if pollen.activated(way.ground) {
                grove.navigate(WAYS[n].href());
            }
        }
    }
}

impl Grown {
    /// Lights the way, or puts it out: its ground comes up under it, its tile turns the green its
    /// patch on the leaf does, and its arrow leans toward where it goes.
    fn light(&self, grove: &mut Grove, lit: bool) {
        let timing = Timing::ms(LIGHT_MS).ease(Ease::Decelerate);
        let (tile, arrow, lean) = match lit {
            true => (theme::LIT, Palette::Ink, 4.0),
            false => (self.hue, QUIET, 0.0),
        };
        grove.animate(self.ground, Motion::Opacity(lit as u8 as f32), timing);
        grove.animate(self.tile, Motion::from(tile), timing);
        grove.animate(self.arrow, Motion::from(arrow), timing);
        grove.animate(self.arrow, Motion::Shift(Position::new(lean, 0.0)), timing);
    }
}

/// Grows way `n` under `above` in `label`, worn in `hue`. Hands back the way, and the parts of it
/// that come in when the label does.
///
/// Its words are placed first and its ground fitted round them, so a name over a line of words
/// that wraps on a narrow label grows a ground as tall as it needs, and moves every way under it.
fn grow_way(
    grove: &mut Grove,
    label: Leaf,
    above: Leaf,
    n: usize,
    way: &Way,
    hue: Color,
    arrow: foliage::Field,
) -> (Grown, Vec<Leaf>) {
    let lead = if n == 0 { LEAD } else { SPACE };
    let size = FontSize::new().xs(NAME[0]).md(NAME[1]).short(NAME[0]);
    let name = grove.branch(
        label,
        Text::new(way.name)
            .color(Palette::Ink)
            .font_size(size)
            .intangible()
            .elevate(Elevation::up(2))
            .anchored(above)
            .at(Location::new().xs(
                left(INDENT.px()).width(content()),
                top(anchor().bottom() + (lead + PAD).px()).height(1.letters()),
            )),
    );
    let says = grove.branch(
        label,
        Text::new(way.says)
            .color(QUIET)
            .font_size(lichen::caption())
            .intangible()
            .elevate(Elevation::up(2))
            .anchored(name)
            .at(Location::new().xs(
                left(INDENT.px()).right(100.pct() - (ARROW + 8.0).px()),
                top(anchor().bottom() + 2.px()).height(content()),
            )),
    );
    // Set at the name's size, so a letter of its own is a line of the name.
    let ground = grove.branch(
        label,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Corners::all(Rounding::Sm))
            .interactive()
            .opacity(0.0)
            .elevate(Elevation::up(1))
            .font_size(size)
            .anchored(says)
            .at(Location::new().xs(
                left(0.px() - PAD.px()).right(100.pct() + PAD.px()),
                top(anchor().top() - 1.letters() - (2.0 + PAD).px())
                    .bottom(anchor().bottom() + PAD.px()),
            )),
    );
    let tile = grove.branch(
        label,
        Polygon::new()
            .sides(6.0)
            .rounding(0.2)
            .color(hue)
            .intangible()
            .elevate(Elevation::up(2))
            .anchored(name)
            .at(Location::new().xs(
                left(0.px()).width(TILE.px()),
                center_y(anchor().top() + anchor().height() * 0.5).height(TILE.px()),
            )),
    );
    let arrow = grove.branch(
        label,
        Icon::new(arrow)
            .color(QUIET)
            .intangible()
            .elevate(Elevation::up(2))
            .anchored(ground)
            .at(Location::new().xs(
                right(100.pct()).width(ARROW.px()),
                center_y(anchor().top() + anchor().height() * 0.5).height(ARROW.px()),
            )),
    );
    let mut parts = vec![name, says, tile, arrow];
    // A rule between one way and the next, from where the words start.
    if n + 1 < WAYS.len() {
        parts.push(grove.branch(
            label,
            Panel::new()
                .color(Palette::Raised)
                .intangible()
                .elevate(Elevation::up(1))
                .anchored(ground)
                .at(Location::new().xs(
                    left(INDENT.px()).right(100.pct()),
                    top(anchor().bottom() + (SPACE / 2.0).px()).height(HAIRLINE.px()),
                )),
        ));
    }
    let grown = Grown {
        ground,
        tile,
        arrow,
        hue,
        lit: false,
    };
    (grown, parts)
}

/// Where the leaf and the label stand, for one size of window.
///
/// From the window's size rather than a placement per breakpoint, because the leaf has to fit both
/// ways -- as large as the room allows, and no taller than the window -- and a width in this
/// grammar may not read a height. Stated again on every resize.
struct Arranged {
    leaf: Location,
    label: Location,
}

impl Arranged {
    fn of(aspect: f32, viewport: Area) -> Self {
        let room = (
            (viewport.width - 2.0 * MARGIN).max(1.0),
            (viewport.height - 2.0 * MARGIN).max(1.0),
        );
        let beside = viewport.width >= BESIDE && viewport.width >= viewport.height * LANDSCAPE;
        match beside {
            // Side by side: the leaf as tall as the window lets it be beside a label as wide as
            // it reads well at, the two centred together, each centred down the window.
            true => {
                let column = (room.0 * 0.36).clamp(COLUMN.0, COLUMN.1);
                let tall = label_height(viewport, column);
                let gap = (room.0 * 0.06).clamp(32.0, 96.0);
                let width = (room.1 * SHARE / aspect)
                    .min(room.0 - column - gap)
                    .max(1.0);
                let height = width * aspect;
                let from = ((viewport.width - (width + gap + column)) / 2.0).max(MARGIN);
                Self {
                    leaf: boxed(from, ((viewport.height - height) / 2.0).max(MARGIN), width, height),
                    label: boxed(
                        from + width + gap,
                        ((viewport.height - tall) / 2.0).max(MARGIN),
                        column,
                        tall,
                    ),
                }
            }
            // One over the other: the leaf as large as leaves the label in view, but never so
            // small it stops reading as a leaf -- the page scrolls instead.
            false => {
                let column = room.0.min(COLUMN.1 + 40.0);
                let tall = label_height(viewport, column);
                let fits = (room.1 - tall - UNDER) / aspect;
                let width = fits
                    .min(room.0)
                    .min(STACKED.0)
                    .max(room.0.min(STACKED.1));
                let height = width * aspect;
                Self {
                    leaf: boxed((viewport.width - width) / 2.0, MARGIN, width, height),
                    label: boxed(
                        (viewport.width - column) / 2.0,
                        MARGIN + height + UNDER,
                        column,
                        tall,
                    ),
                }
            }
        }
    }
}

/// A box `width` by `height`, its corner `x` across and `y` down.
fn boxed(x: f32, y: f32, width: f32, height: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(height.px()),
    )
}

/// About how tall the label stands on a window of `viewport`, `column` wide: what it is centred by,
/// and what the leaf over it leaves room for. Content past it still counts for how far the page
/// scrolls, so an estimate a line short costs a little scrolling and nothing else.
fn label_height(viewport: Area, column: f32) -> f32 {
    // Short by its entering height: past it the sizes may still be short, in the deadband, and the
    // estimate is a line or so long for a frame, which costs nothing.
    let layout = match viewport.height < Short::ENTER {
        true => Layout::Xs,
        false => Layout::of(viewport),
    };
    let wordmark = match layout {
        Layout::Xs | Layout::Sm => WORDMARK[0],
        Layout::Md => WORDMARK[1],
        Layout::Lg | Layout::Xl => WORDMARK[2],
    } as f32;
    let name = match layout {
        Layout::Xs | Layout::Sm => NAME[0],
        _ => NAME[1],
    } as f32;
    let caption = lichen::measure().caption as f32;
    let says = column - INDENT - ARROW - 8.0;
    let ways: f32 = WAYS
        .iter()
        .map(|way| 2.0 * PAD + CELL * name + 2.0 + CELL * caption * lines(way.says, caption, says))
        .sum();
    let between = (WAYS.len() - 1) as f32 * SPACE;
    CELL * wordmark + 4.0 + CELL * caption * lines(TAGLINE, caption, column) + LEAD + ways + between
}

/// How many lines `text` wraps to at `size`, `width` wide.
fn lines(text: &str, size: f32, width: f32) -> f32 {
    let wide = text.chars().count() as f32 * ADVANCE * size;
    (wide / width.max(1.0)).ceil().max(1.0)
}
