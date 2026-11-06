//! The page: the leaf, and the label beside it.
//!
//! The leaf is mounted the way a herbarium sheet mounts a specimen, and the label is what a sheet's
//! corner carries: what it is, and where the rest of the collection is kept -- the three apps
//! grown on foliage and its source, and, under a heading of their own, where it is written down.
//!
//! Light comes through the leaf. Once it has formed, a soft light rises in it and wanders slowly
//! over the blade, paling the tiles it crosses, and pressing the leaf draws the light to where it
//! was pressed -- the same on a phone as with a mouse, since nothing on the page waits on being
//! pointed at.
//!
//! On a window wide enough the two stand side by side, centred as a pair; on a narrow one the leaf
//! stands over the label, as large as leaves the label in view. Both are placements of the same
//! things, stated again whenever the window is resized, and the page scrolls only where a window is
//! too short for either.
//!
//! How tall the label is, is the engine's answer rather than the page's: the label is as tall as
//! what it holds, each line placed under the last, and what the arrangement centres it by and leaves
//! room for is the height it measured to.

use core::f32::consts::FRAC_PI_3;

use foliage::{
    Area, Axes, Boxed, Corners, Ease, Elevation, Field, FontSize, Grove, Grow, HAIRLINE, Icon,
    Leaf, Location, Motion, Palette, Panel, Place, Pollen, Polygon, Position, Root, Rounding, Sap,
    Shape, Source, Stem, Text, Timing, Vein, anchor, center_y, content, left, right, top,
};
use lichen::{Mosaic, Ramp, Scatter};
use tracing::info;

use crate::icons::Icons;
use crate::leaf::{self, Sun};
use crate::theme::{self, MARGIN};

/// One way off the page.
struct Way {
    name: &'static str,
    /// What is there, said under its name.
    says: &'static str,
    /// Where it is, from the site's own root on the web -- which holds whether or not the address
    /// the site was read from ends in a slash -- or outright.
    at: Where,
    mark: Mark,
}

enum Where {
    /// A page of the site's own, by its path from the site's root.
    Here(&'static str),
    /// Somewhere else altogether.
    Away(&'static str),
}

/// What a way is marked with in the label.
#[derive(Copy, Clone)]
enum Mark {
    /// A tile worn in its own stretch of the leaf's ramp.
    Tile(usize),
    /// The kind of writing it leads to.
    Guide,
    Reference,
}

/// Where the site is published.
const PUBLISHED: &str = "https://eblack-leaf.github.io";

/// The apps grown on foliage, and its source.
const GROWN: [Way; 4] = [
    Way {
        name: "conservatory",
        says: "an exhibition of grown works",
        at: Where::Here("/foliage/portfolio/conservatory/"),
        mark: Mark::Tile(0),
    },
    Way {
        name: "orchard",
        says: "a boutique of digital goods",
        at: Where::Here("/foliage/portfolio/orchard/"),
        mark: Mark::Tile(1),
    },
    Way {
        name: "trellis",
        says: "a planning sheet for a team",
        at: Where::Here("/foliage/portfolio/trellis/"),
        mark: Mark::Tile(2),
    },
    Way {
        name: "github",
        says: "eblack-leaf/foliage",
        at: Where::Away("https://github.com/eblack-leaf/foliage"),
        mark: Mark::Tile(3),
    },
];

/// Where foliage is written down.
const DOCS: [Way; 2] = [
    Way {
        name: "book",
        says: "the guide: how an app is put together",
        at: Where::Here("/foliage/book/"),
        mark: Mark::Guide,
    },
    Way {
        name: "api",
        says: "the reference: every type and verb",
        at: Where::Here("/foliage/api/foliage/"),
        mark: Mark::Reference,
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

/// How a set of ways is set: the apps large and ruled apart, the docs smaller and closer, the way
/// an index is.
struct Set {
    /// Its names' size at `xs`, and at `md` and wider.
    name: [u32; 2],
    /// How far a way's ground reaches past its words, above and below.
    pad: f32,
    /// The room before its first way, and between one way's ground and the next.
    lead: f32,
    space: f32,
    /// Whether a rule stands between one way and the next.
    ruled: bool,
}

const APPS: Set = Set {
    name: [17, 19],
    pad: 12.0,
    lead: 26.0,
    space: 6.0,
    ruled: true,
};

const INDEX: Set = Set {
    name: [14, 15],
    pad: 8.0,
    lead: 6.0,
    space: 2.0,
    ruled: false,
};

/// What the label says under the wordmark, and over the docs.
const TAGLINE: &str = "a ui library for rust, and three apps grown on it";
const HEADING: &str = "docs";

/// The room between the last of the apps and the docs' heading.
const SECTION: f32 = 30.0;

/// The wordmark's size at `xs`, `md` and `lg`. A window too short for it takes the `xs` size
/// whatever its width, and the names do the same.
const WORDMARK: [u32; 3] = [34, 44, 52];

/// What quiet words are read in: what is there, under a way's name, the tagline and the heading.
/// Ink, a step into the ground, rather than the muted role, which is a hint's colour and too faint
/// to read whole lines in.
const QUIET: Palette = Palette::Ink.recede();

/// How far in from the label's edge a way's words start, past its mark.
const INDENT: f32 = 28.0;

/// How large a way's tile is, a doc's mark, and the arrow at a way's end.
const TILE: f32 = 13.0;
const GLYPH: f32 = 15.0;
const ARROW: f32 = 16.0;

/// How long a way takes to light, or to go out, and how long its tile takes to turn.
const LIGHT_MS: u64 = 220;
const TURN_MS: u64 = 520;

/// A way's tile, a hexagon, and how far it turns while lit: a sixth, which lands it on itself.
const HEXAGON: Shape = Shape {
    sides: 6.0,
    rounding: 0.2,
    rotation: 0.0,
};
const TURN: f32 = FRAC_PI_3;

/// How often the light is moved: about twenty-five times a second, which a light crossing a few
/// tiles a second needs and a phone's battery would rather not be asked for more of.
const DRIFT_MS: u64 = 40;

/// How wide the label is, at least and at most.
const COLUMN: (f32, f32) = (300.0, 420.0);

/// The narrowest window the two stand side by side on, and how much wider than tall it has to be.
const BESIDE: f32 = 720.0;
const LANDSCAPE: f32 = 0.9;

/// How much of a window's height the leaf takes, beside the label.
const SHARE: f32 = 0.94;

/// How wide the leaf is at most, and at least, over the label.
const STACKED: (f32, f32) = (440.0, 200.0);

/// The room between the leaf and the label, over it.
const UNDER: f32 = 28.0;

/// The app.
pub(crate) struct Site {
    mosaic: Mosaic,
    /// What the leaf is grown in, placed again on every resize.
    leaf: Leaf,
    /// What the label is grown in, placed again on every resize.
    label: Leaf,
    /// How tall what the label holds measured, as the arrangement last used it.
    tall: f32,
    aspect: f32,
    /// What takes a press on the leaf, over the whole of it.
    touch: Leaf,
    ways: Vec<Grown>,
    /// The light through the leaf.
    sun: Sun,
}

/// One way, grown.
struct Grown {
    /// The one element of it that takes a press, over the whole of it.
    press: Leaf,
    /// What comes up under its words while it is lit.
    ground: Leaf,
    mark: Leaf,
    /// Whether its mark is a tile, which turns while it is lit, or a glyph, which brightens.
    tile: bool,
    arrow: Leaf,
    href: String,
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
        // Nothing is measured before the first frame, so the first arrangement is a guess the
        // second puts right -- before the label has started to come in.
        let arranged = Arranged::of(aspect, grove.viewport(), 0.0);
        let shape = mosaic.grow(grove, page, arranged.leaf, &mut Scatter::new());
        mosaic.form(grove);
        // Over the tiles and the outline both, which would otherwise take a press themselves.
        let touch = grove.branch(
            shape,
            Stem::new()
                .interactive()
                .elevate(Elevation::up(3))
                .at(Location::new()),
        );

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
                .at(Location::new()
                    .xs(left(0.px()).width(content()), top(0.px()).height(content()))),
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
        let mut ways = Vec::new();
        let above = grow_set(
            grove,
            label,
            tagline,
            &GROWN,
            &APPS,
            &icons,
            &mut ways,
            &mut arrivals,
        );
        // The docs, set apart: a heading with a rule run out from it to the label's edge, and an
        // index under it.
        let heading = grove.branch(
            label,
            Text::new(HEADING)
                .color(QUIET)
                .font_size(lichen::caption())
                .intangible()
                .anchored(above)
                .at(Location::new().xs(
                    left(0.px()).width(content()),
                    top(anchor().bottom() + SECTION.px()).height(1.letters()),
                )),
        );
        let rule = grove.branch(
            label,
            Panel::new()
                .color(Palette::Raised)
                .intangible()
                .anchored(heading)
                .at(Location::new().xs(
                    left(anchor().right() + 10.px()).right(100.pct()),
                    center_y(anchor().top() + anchor().height() * 0.5).height(HAIRLINE.px()),
                )),
        );
        arrivals.push(vec![heading, rule]);
        grow_set(
            grove,
            label,
            heading,
            &DOCS,
            &INDEX,
            &icons,
            &mut ways,
            &mut arrivals,
        );
        // The label comes in a line at a time while the leaf is still forming, so the ways off
        // the page are there to press without waiting on it.
        for (n, parts) in arrivals.iter().enumerate() {
            let timing = Timing::ms(480)
                .after(240 + 80 * n as u64)
                .ease(Ease::Decelerate);
            for &part in parts {
                grove.opacity(part, 0.0);
                grove.shift(part, Position::new(0.0, 8.0));
                grove.animate(part, Motion::Opacity(1.0), timing);
                grove.animate(part, Motion::Shift(Position::default()), timing);
            }
        }
        info!("leaf planted");
        Site {
            mosaic,
            leaf: shape,
            label,
            tall: 0.0,
            aspect,
            touch,
            ways,
            sun: Sun::new(),
        }
    }

    fn frame(&mut self, grove: &mut Grove, pollen: Pollen) {
        // Arranged again when the window changes, and when what the label holds measured to
        // something else -- which a resize that rewraps it does a frame later, and the first frame
        // does once. The leaf's size reads the label's height, which no width in the grammar may,
        // so that second look is the page's own -- asked for at once, so a rewrap is put right on
        // the very next frame rather than whenever the next one happens to come.
        let tall = match grove.tap(self.label, Vein::Placed) {
            Some(Sap::Section(placed)) => placed.height() - MARGIN,
            _ => self.tall,
        };
        let resized = pollen.resized().is_some();
        if resized || (tall - self.tall).abs() > 0.5 {
            self.tall = tall;
            let arranged = Arranged::of(self.aspect, grove.viewport(), tall);
            grove.at(self.leaf, arranged.leaf);
            grove.at(self.label, arranged.label);
            if resized {
                grove.again();
            }
        }
        self.mosaic.frame(grove, &pollen);
        // The light, from once the leaf has formed: drawn to where the leaf was pressed, or to
        // its middle by a key, and always moving -- slowly enough that a frame every `DRIFT_MS`
        // carries it, and the page sleeps in between rather than drawing at the display's rate.
        if self.mosaic.formed(&pollen) {
            let pressed = match pollen.clicked_at(self.touch) {
                Some(at) => self.mosaic.point(grove, at),
                None if pollen.activated(self.touch) => Some((0.5, 0.66)),
                None => None,
            };
            let (at, strength) = self.sun.frame(grove.elapsed().as_secs_f32(), pressed);
            self.mosaic.shine(grove, at, leaf::REACH, strength);
            grove.again_in(DRIFT_MS);
        }
        // Lit while the cursor rests on it or focus does: the one a press would reach, as the
        // engine reads it, so nothing is lit through whatever covers it.
        let (hovered, focused) = (grove.hovered(), grove.focused());
        for way in &mut self.ways {
            let lit = hovered == Some(way.press) || focused == Some(way.press);
            if lit != way.lit {
                way.lit = lit;
                way.light(grove, lit);
            }
            if pollen.activated(way.press) {
                grove.navigate(way.href.clone());
            }
        }
    }
}

impl Grown {
    /// Lights the way, or puts it out: its ground comes up under it, its mark turns -- a tile a
    /// sixth of the way round, a glyph to ink -- and its arrow leans toward where it goes.
    fn light(&self, grove: &mut Grove, lit: bool) {
        let timing = Timing::ms(LIGHT_MS).ease(Ease::Decelerate);
        let (arrow, lean) = match lit {
            true => (Palette::Ink, 4.0),
            false => (QUIET, 0.0),
        };
        grove.animate(self.ground, Motion::Opacity(lit as u8 as f32), timing);
        match self.tile {
            true => {
                let turned = Shape {
                    rotation: if lit { TURN } else { 0.0 },
                    ..HEXAGON
                };
                grove.animate(
                    self.mark,
                    Motion::Polygon(turned),
                    Timing::ms(TURN_MS).ease(Ease::Emphasis),
                );
            }
            false => {
                let glyph = if lit { Palette::Ink } else { QUIET };
                grove.animate(self.mark, Motion::from(glyph), timing);
            }
        }
        grove.animate(self.arrow, Motion::from(arrow), timing);
        grove.animate(self.arrow, Motion::Shift(Position::new(lean, 0.0)), timing);
    }
}

/// Grows `set`'s ways under `above` in `label`, one under the other, into `ways`, and what comes in
/// with each into `arrivals`. Hands back the last way's press, for what stands under the set.
#[allow(clippy::too_many_arguments)]
fn grow_set(
    grove: &mut Grove,
    label: Leaf,
    above: Leaf,
    set: &[Way],
    setting: &Set,
    icons: &Icons,
    ways: &mut Vec<Grown>,
    arrivals: &mut Vec<Vec<Leaf>>,
) -> Leaf {
    let mut above = above;
    for (n, way) in set.iter().enumerate() {
        let lead = if n == 0 { setting.lead } else { setting.space };
        let ruled = setting.ruled && n + 1 < set.len();
        let (grown, parts) = grow_way(grove, label, above, lead, ruled, way, setting, icons);
        above = grown.press;
        ways.push(grown);
        arrivals.push(parts);
    }
    above
}

/// Grows `way`, `lead` under `above` in `label`, set as `setting` says, with a rule under it if it
/// is `ruled`. Hands back the way, and the parts of it that come in when the label does.
///
/// Its words are placed first and its ground fitted round them, so a name over a line of words
/// that wraps on a narrow label grows a ground as tall as it needs, and moves every way under it.
#[allow(clippy::too_many_arguments)]
fn grow_way(
    grove: &mut Grove,
    label: Leaf,
    above: Leaf,
    lead: f32,
    ruled: bool,
    way: &Way,
    setting: &Set,
    icons: &Icons,
) -> (Grown, Vec<Leaf>) {
    let pad = setting.pad;
    let size = FontSize::new()
        .xs(setting.name[0])
        .md(setting.name[1])
        .short(setting.name[0]);
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
                top(anchor().bottom() + (lead + pad).px()).height(1.letters()),
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
    // The press is a stem over the whole way rather than the ground it shows: foliage leaves a
    // fully transparent element out of the box stack, so a ground at rest is not there to press,
    // or to be pointed at. Set at the name's size, so a letter of its own is a line of the name.
    let press = grove.branch(
        label,
        Stem::new()
            .interactive()
            .elevate(Elevation::up(1))
            .font_size(size)
            .anchored(says)
            .at(Location::new().xs(
                left(0.px() - 12.px()).right(100.pct() + 12.px()),
                top(anchor().top() - 1.letters() - (2.0 + pad).px())
                    .bottom(anchor().bottom() + pad.px()),
            )),
    );
    let ground = grove.branch(
        press,
        Panel::new()
            .color(Palette::Raised)
            .rounding(Corners::all(Rounding::Sm))
            .intangible()
            .opacity(0.0)
            .at(Location::new()),
    );
    // On the name's line, at the label's edge.
    let on_name = |side: f32| {
        Location::new().xs(
            left(0.px()).width(side.px()),
            center_y(anchor().top() + anchor().height() * 0.5).height(side.px()),
        )
    };
    let mark = match way.mark {
        Mark::Tile(n) => grove.branch(
            label,
            Polygon::new()
                .sides(HEXAGON.sides)
                .rounding(HEXAGON.rounding)
                .color(theme::HUES[n])
                .intangible()
                .elevate(Elevation::up(2))
                .anchored(name)
                .at(on_name(TILE)),
        ),
        Mark::Guide | Mark::Reference => {
            let field: Field = match way.mark {
                Mark::Guide => icons.book_open,
                _ => icons.file_text,
            };
            grove.branch(
                label,
                Icon::new(field)
                    .color(QUIET)
                    .intangible()
                    .elevate(Elevation::up(2))
                    .anchored(name)
                    .at(on_name(GLYPH)),
            )
        }
    };
    let arrow = grove.branch(
        label,
        Icon::new(icons.arrow_right)
            .color(QUIET)
            .intangible()
            .elevate(Elevation::up(2))
            .anchored(press)
            .at(Location::new().xs(
                right(100.pct()).width(ARROW.px()),
                center_y(anchor().top() + anchor().height() * 0.5).height(ARROW.px()),
            )),
    );
    let mut parts = vec![name, says, mark, arrow];
    // A rule between one way and the next, from where the words start.
    if ruled {
        parts.push(
            grove.branch(
                label,
                Panel::new()
                    .color(Palette::Raised)
                    .intangible()
                    .elevate(Elevation::up(1))
                    .anchored(press)
                    .at(Location::new().xs(
                        left(INDENT.px()).right(100.pct()),
                        top(anchor().bottom() + (setting.space / 2.0).px()).height(HAIRLINE.px()),
                    )),
            ),
        );
    }
    let grown = Grown {
        press,
        ground,
        mark,
        tile: matches!(way.mark, Mark::Tile(_)),
        arrow,
        href: way.href(),
        lit: false,
    };
    (grown, parts)
}

/// Where the leaf and the label stand, for one size of window.
///
/// From the window's size rather than a placement per breakpoint, because the leaf has to fit both
/// ways -- as large as the room allows, and no taller than the window -- and a width in this
/// grammar may not read a height. Stated again on every resize, and whenever the label measures to
/// another height.
struct Arranged {
    leaf: Location,
    label: Location,
}

impl Arranged {
    /// For a label whose contents measured `tall`.
    fn of(aspect: f32, viewport: Area, tall: f32) -> Self {
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
                let gap = (room.0 * 0.06).clamp(32.0, 96.0);
                let width = (room.1 * SHARE / aspect)
                    .min(room.0 - column - gap)
                    .max(1.0);
                let height = width * aspect;
                let from = ((viewport.width - (width + gap + column)) / 2.0).max(MARGIN);
                Self {
                    leaf: boxed(
                        from,
                        ((viewport.height - height) / 2.0).max(MARGIN),
                        width,
                        height,
                    ),
                    label: hugging(
                        from + width + gap,
                        ((viewport.height - tall) / 2.0).max(MARGIN),
                        column,
                    ),
                }
            }
            // One over the other: the leaf as large as leaves the label in view, but never so
            // small it stops reading as a leaf -- the page scrolls instead.
            false => {
                let column = room.0.min(COLUMN.1 + 40.0);
                let fits = (room.1 - tall - UNDER) / aspect;
                let width = fits.min(room.0).min(STACKED.0).max(room.0.min(STACKED.1));
                let height = width * aspect;
                Self {
                    leaf: boxed((viewport.width - width) / 2.0, MARGIN, width, height),
                    label: hugging(
                        (viewport.width - column) / 2.0,
                        MARGIN + height + UNDER,
                        column,
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

/// A box `width` wide, its corner `x` across and `y` down, as tall as what it holds -- and a margin
/// more, so a page short enough to scroll scrolls to a margin past it.
fn hugging(x: f32, y: f32, width: f32) -> Location {
    Location::new().xs(
        left(x.px()).width(width.px()),
        top(y.px()).height(content() + MARGIN.px()),
    )
}

#[cfg(test)]
mod tests {
    use foliage::{Area, Greenhouse, Sap, Vein};

    use super::{STACKED, Site, UNDER};
    use crate::theme::MARGIN;

    /// The site grown at `viewport` and run until it has settled.
    fn grown(viewport: Area) -> Greenhouse<Site> {
        let mut house = Greenhouse::<Site>::new(viewport);
        house.frame();
        house.settle();
        // Saved where asked, to look at.
        if let Ok(dir) = std::env::var("SITE_SHOTS")
            && let Some(picture) = house.render()
        {
            let name = format!("{}x{}.png", viewport.width, viewport.height);
            std::fs::write(std::path::Path::new(&dir).join(name), picture.png()).unwrap();
        }
        house
    }

    fn placed(house: &Greenhouse<Site>, leaf: foliage::Leaf) -> foliage::Section {
        match house.grove().tap(leaf, Vein::Placed) {
            Some(Sap::Section(section)) => section,
            other => panic!("expected a section, got {other:?}"),
        }
    }

    /// The label is as tall as what it holds, at either shape of window.
    #[test]
    fn the_label_is_as_tall_as_what_it_holds() {
        for viewport in [Area::new(1280.0, 800.0), Area::new(390.0, 844.0)] {
            let house = grown(viewport);
            let site = house.root();
            assert!(site.tall > 200.0, "{viewport:?}: measured {}", site.tall);
            let label = placed(&house, site.label);
            assert!(
                (label.height() - (site.tall + MARGIN)).abs() < 0.01,
                "{viewport:?}: {label:?}"
            );
        }
    }

    /// One over the other, the leaf leaves the label its measured height in view, and the label
    /// stands under it.
    #[test]
    fn over_the_label_the_leaf_leaves_it_room() {
        let viewport = Area::new(390.0, 844.0);
        let house = grown(viewport);
        let site = house.root();
        let (leaf, label) = (placed(&house, site.leaf), placed(&house, site.label));
        assert_eq!(label.top(), leaf.bottom() + UNDER);
        // Narrow enough that the leaf is at its least, or the label ends inside the window.
        assert!(
            leaf.width() <= STACKED.1 + 0.5
                || label.top() + site.tall <= viewport.height - MARGIN + 0.5,
            "{leaf:?}, {label:?}, {}",
            site.tall
        );
    }

    /// A resize that rewraps the label is put right on the next frame: the one it lands in is
    /// arranged for the old height, and asks for the one after, which is arranged for the new.
    #[test]
    fn a_resize_that_rewraps_the_label_is_put_right_a_frame_later() {
        let mut house = grown(Area::new(1280.0, 800.0));
        let wide = house.root().tall;
        let viewport = Area::new(1000.0, 800.0);
        house.resize(viewport);
        // Only frames that are owed: the second is there because the first asked for it.
        house.settle();
        let site = house.root();
        assert!((site.tall - wide).abs() > 0.5, "rewrapped: {wide} to {}", site.tall);
        let label = placed(&house, site.label);
        assert!((label.height() - (site.tall + MARGIN)).abs() < 0.01, "{label:?}");
        let centred = ((viewport.height - site.tall) / 2.0).max(MARGIN);
        assert!((label.top() - centred).abs() < 0.5, "{label:?} against {centred}");
    }

    /// Side by side, the label is centred down the window by the height it measured.
    #[test]
    fn beside_the_leaf_the_label_is_centred_by_its_height() {
        let viewport = Area::new(1280.0, 800.0);
        let house = grown(viewport);
        let site = house.root();
        let label = placed(&house, site.label);
        let centred = ((viewport.height - site.tall) / 2.0).max(MARGIN);
        assert!(
            (label.top() - centred).abs() < 0.5,
            "{label:?} against {centred}"
        );
    }
}
