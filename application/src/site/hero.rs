//! The opener: a full-bleed first screen, then a hint to keep going.
//!
//! Nothing types itself in and nothing spins. Shapes resolve in place -- rough polygons
//! becoming their real form -- which reads as arrival rather than as travel.

use foliage::{
    Bare, Forest, Color, Ease, Elevation, FontSize, GlyphColors, Grid, GridExt, Grows,
    HorizontalAlignment, Icon, IconId, Layout, Leaf, Location, Motion, Repeat, Sprout, Text,
    VerticalAlignment, anchor, text_content,
};

use crate::icons::IconHandles;
use crate::site::copy::hero::{
    AT_BREAKPOINT, AT_H, AT_MS, AT_SHORT, AT_W, EXTENSION_AT, HINT as HINT_TEXT, TAGLINE, WORDMARK,
    hud_line,
};
use crate::site::{
    Grow, POLY_BUTTON_ROW_H, PolyButton, fade_in, poly_button, role, space, timing, type_scale,
};

/// The project's namesake, so it takes as much room as each viewport can give it. Ten
/// monospace characters run about 6x the font size, which is what caps each step -- 40px
/// overflows a 360px phone, which is what clipped the first attempt.
const WORDMARK_XS: u32 = 40;
const WORDMARK_MD: u32 = 76;
/// Landscape is `md`-wide, so it would otherwise take `WORDMARK_MD` in a ~250px-tall viewport.
///
/// `short` is a single size with no width steps under it -- it wins over the whole `xs`/`md`
/// chain -- so this one number has to survive the narrowest window that is also short. The
/// wordmark gets `SHORT_TEXT_RIGHT_PCT` of the width there, and ten monospace characters run
/// about 6x the font size, so 22 needs ~132px of column -- which a 260px-wide window still
/// has. Short *and* `xs` is the case that sets this: at 30 the name lost its last character
/// on anything under ~350px, and 26 only moved that to ~300.
const WORDMARK_SHORT: u32 = 22;

/// The buttons live in their own row so `max` has something to constrain. Positioned as
/// thirds of the *hero*, they drifted to the far corners of a wide monitor -- a max on the
/// hero could never fix that, because the hero is supposed to be full width.
const ROW_MAX: f32 = 420.0;

/// The opening beat, in ms. Ordered rather than staggered by a single step: the name lands
/// first and alone, then what it is, and only then do the destinations resolve.
const AT_WORDMARK: u64 = 0;
const AT_TAGLINE: u64 = 320;
const AT_BUTTONS: u64 = 700;
/// Between one button and the next. Wide enough to watch each shape become itself, which is
/// the whole reason they are three different shapes.
const BUTTON_STEP: u64 = 300;
const AT_READOUT: u64 = 520;
const AT_HINT: u64 = AT_BUTTONS + BUTTON_STEP * 3 + 260;
/// Short viewports get the hero side by side instead of stacked: wordmark and tagline on
/// the left, destinations on the right. Stacked, the wordmark, buttons and the fold hint
/// all land on top of each other -- there simply is not 100% of a 250px-tall viewport to
/// share out. Landscape has the opposite budget, so the layout turns ninety degrees.
const SHORT_TEXT_RIGHT_PCT: f32 = 52.0;
const SHORT_BUTTONS_LEFT_PCT: f32 = 52.0;
const SHORT_BUTTONS_TOP_PCT: f32 = 46.0;

const ROW_H: i32 = 16;
/// One step down from `LABEL`, not two. 39 characters at ~0.6em each is ~281px at 12 and
/// ~257px at 11 -- enough to keep the line on one row on a narrow window without it reading as
/// a different, smaller thing than it is everywhere else.
const HUD_SHORT: u32 = 11;

/// Which glyphs are lit. Rebuilt alongside the string rather than fixed at spawn, since the
/// step that is lit is itself a reading.
fn hud_colors(layout: Layout, short: bool) -> GlyphColors {
    // One colour per metric, taken from the three destination buttons directly below. Three
    // readings in one tone read as one number that happens to have spaces in it; borrowing the
    // row's own palette makes them three things and costs the page no new hues.
    let size = Color::amber(400);
    let frame = Color::rose(400);
    let breakpoint = role::accent();
    // Dimmer than the separators, so an unlit step reads as part of a scale rather than as a
    // word someone left there. The `x` and the `ms` unit stay at the base tone.
    let unlit = Color::stone(700);
    let step = match layout {
        Layout::Xs => 0,
        Layout::Sm => 1,
        Layout::Md => 2,
        Layout::Lg => 3,
        Layout::Xl => 4,
    };
    let mut colors = GlyphColors::new()
        .add(AT_W, size)
        .add(AT_H, size)
        .add(AT_MS, frame)
        .add(AT_SHORT, if short { breakpoint } else { unlit });
    for (i, span) in AT_BREAKPOINT.iter().enumerate() {
        colors = colors.add(span.clone(), if i == step { breakpoint } else { unlit });
    }
    colors
}

/// Weight of each new frame-time sample against the running value. It jitters by a millisecond
/// or two every frame, and an unsmoothed readout strobes through digits too fast to read --
/// which looks like decoration rather than a measurement.
const SMOOTHING: f32 = 0.06;
/// How often the *displayed* value is allowed to change.
///
/// Smoothing alone was not enough: the average still moves a tenth of a millisecond most
/// frames, so the last digit flickered continuously and the eye went straight to it. The value
/// keeps averaging every frame; only the string it renders to is held. Twice a second is fast
/// enough to read as live and slow enough to actually read.
const REFRESH: f32 = 0.5;

/// The live line.
///
/// Driven from the frame: every reading it shows -- the window's size, how long the last frame
/// took, which breakpoint is in force -- is something the frame can sample directly, so there
/// is nothing for the engine to run on the app's behalf.
pub(crate) struct Readout {
    leaf: Leaf,
    ms: f32,
    since: f32,
    shown: Option<String>,
    /// Which step was last lit, so the colour map is only rebuilt when it actually moves.
    lit: Option<(Layout, bool)>,
}

impl Readout {
    pub(crate) fn drive(&mut self, forest: &mut Forest) {
        let dt = forest.frame_time().as_secs_f32();
        let frame_ms = dt * 1000.0;
        // first sample lands whole rather than easing up from zero, or the number spends its
        // first second visibly wrong
        let first = self.ms == 0.0;
        self.ms = if first {
            frame_ms
        } else {
            self.ms + (frame_ms - self.ms) * SMOOTHING
        };
        // averaging runs every frame; the *rendered* value is held between refreshes
        self.since += dt;
        if !first && self.since < REFRESH {
            return;
        }
        self.since = 0.0;
        let viewport = forest.viewport();
        let line = hud_line(viewport.width(), viewport.height(), self.ms);
        // repaint only on a real change -- a text write re-shapes the whole string, and most
        // refreshes round to the same digits
        if self.shown.as_deref() != Some(line.as_str()) {
            self.shown = Some(line.clone());
            forest.text(self.leaf, line);
        }
        // colours change far less often than the string, and only ever together -- one write
        // when the breakpoint moves, not one every refresh
        let lit = (forest.layout(), forest.short());
        if self.lit != Some(lit) {
            self.lit = Some(lit);
            forest.glyph_colors(self.leaf, hud_colors(lit.0, lit.1));
        }
    }
}

/// The word's own box. Shared with the hit band, which derives its top from it -- as a literal
/// in the label's `Location` it was invisible to the band, and the band guessed wrong.
const HINT_H: i32 = 24;
const CHEVRON: i32 = 22;
/// A breath, not a bounce.
const BOB_PX: i32 = 8;
const BOB_MS: u64 = 900;

/// Warm set rather than one cool outlier -- amber through rose reads as a family the
/// sand/orange scheme belongs to.
fn destinations() -> [PolyButton; 3] {
    [
        PolyButton {
            label: "docs",
            icon: IconHandles::Code,
            href: "https://eblack-leaf.github.io/foliage/api/foliage/index.html",
            sides: 7.0,
            face: Color::amber(400),
        },
        PolyButton {
            label: "book",
            icon: IconHandles::BookOpen,
            href: "https://eblack-leaf.github.io/foliage/book/",
            sides: 6.0,
            face: role::accent(),
        },
        PolyButton {
            label: "github",
            icon: IconHandles::Github,
            href: "https://github.com/eblack-leaf/foliage",
            sides: 5.0,
            face: Color::rose(400),
        },
    ]
}

/// The hero is its own route, and the only one without the rail.
///
/// It was the top of the overview page, which put two scroll contexts back to back and left
/// the rail sitting beside a landing screen it has nothing to do with. As a route it owns
/// the whole viewport, and `more` navigates rather than scrolls -- which is what people try
/// to click anyway.
pub(crate) fn build(g: &mut Grow, slot: Leaf) {
    let seq = g.forest.sequence();
    let hero = g.forest.branch(
        slot,
        Bare::new()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .pass_through(),
    );

    let wordmark = g.forest.branch(
        hero,
        Text::new(WORDMARK)
            .size(
                FontSize::new(WORDMARK_XS)
                    .md(WORDMARK_MD)
                    .short(WORDMARK_SHORT),
            )
            .color(role::on_surface())
            .glyph_colors(|i| {
                if i >= EXTENSION_AT {
                    role::accent()
                } else {
                    role::on_surface()
                }
            })
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    30.pct().as_center_y().with(1.letters().as_height()),
                )
                .short(
                    0.pct()
                        .as_left()
                        .with(SHORT_TEXT_RIGHT_PCT.pct().as_right()),
                    38.pct().as_center_y().with(1.letters().as_height()),
                ))
            .elevate(Elevation::up(3))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .opacity(0.0),
    );
    fade_in(g.forest, wordmark, seq, AT_WORDMARK);

    let tagline = g.forest.branch(
        hero,
        Text::new(TAGLINE)
            .size(FontSize::new(type_scale::BODY))
            .color(role::on_surface_variant())
            .at(Location::new()
                // A step more than a nudge, and no more than that. The wordmark's box is a
                // single letter-height, so its descenders sit close to the bottom edge and 16px
                // read as the two lines touching -- but the tagline hangs off the wordmark in
                // px while the buttons sit at a percentage, so every px added here eats into a
                // gap that is already smallest on the shortest window that is not `short`.
                // The height is the run's own: it grows to whatever it wraps to instead of being
                // scissored, since the fixed height it used to carry cut the second line off in
                // landscape.
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    anchor()
                        .bottom()
                        .as_top()
                        .adjust(space::LG)
                        .with(text_content().as_height()),
                )
                .short(
                    0.pct()
                        .as_left()
                        .with(SHORT_TEXT_RIGHT_PCT.pct().as_right()),
                    anchor()
                        .bottom()
                        .as_top()
                        .adjust(space::MD)
                        .with(text_content().as_height()),
                ))
            .elevate(Elevation::up(3))
            .align(HorizontalAlignment::Center, VerticalAlignment::Top)
            .anchored(wordmark)
            .opacity(0.0),
    );
    fade_in(g.forest, tagline, seq, AT_TAGLINE);
    readout(g, hero, wordmark, seq);

    // Capped and centred, so three buttons stay a group instead of drifting to the corners.
    //
    // Stacked under the tagline rather than pinned to a percentage of the hero. The tagline
    // hangs off the wordmark in px and wraps to two lines when it is narrow, while a
    // percentage does not move for either -- so the two were guaranteed to meet at some
    // height, and did, just above the `short` threshold. Chaining costs the row its place on a
    // very tall window, where it now sits higher than a percentage would put it. That is the
    // cheaper of the two: too high is a look, overlapping is a bug.
    let row = g.forest.branch(
        hero,
        Bare::new()
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()).max(ROW_MAX),
                    anchor()
                        .bottom()
                        .as_top()
                        .adjust(space::XL)
                        .with(POLY_BUTTON_ROW_H.px().as_height()),
                )
                .short(
                    SHORT_BUTTONS_LEFT_PCT
                        .pct()
                        .as_left()
                        .with(100.pct().as_right())
                        .max(ROW_MAX),
                    SHORT_BUTTONS_TOP_PCT
                        .pct()
                        .as_center_y()
                        .with(POLY_BUTTON_ROW_H.px().as_height()),
                ))
            .elevate(Elevation::up(1))
            .grid(Grid::new(1.col().gap(0), 1.row().gap(0)))
            .anchored(tagline)
            .pass_through(),
    );
    let specs = destinations();
    let third = 100.0 / specs.len() as f32;
    for (i, spec) in specs.iter().enumerate() {
        let center = third * i as f32 + third / 2.0;
        poly_button(
            g,
            row,
            spec,
            center,
            seq,
            AT_BUTTONS + BUTTON_STEP * i as u64,
        );
    }
    hint(g, hero, seq);
}

/// One line of live readings, sitting directly above the wordmark.
///
/// One line, deliberately. A block of key/value rows is a fixed height that has to be fitted
/// somewhere, and this screen has nowhere for it -- under the tagline it crowded the line
/// above it, above the wordmark it ran off the top, and clearing a path pushed the button row
/// into the fold hint. Scattering the rows into the margins instead only traded that for four
/// unrelated texts floating in the corners. A single line has no block to place and nothing to
/// read as a set: it is a caption on the name.
///
/// Every field changes when the window does, which is why there are three and not six. An
/// `entities` count that never moves on a static page, or a `renderer` that is always `wgpu`,
/// is decoration wearing a measurement's clothes -- the whole point of putting real numbers
/// here is that they are doing something.
///
/// Parked on `short`, where the hero has already spent its height.
fn readout(g: &mut Grow, hero: Leaf, wordmark: Leaf, seq: Leaf) {
    let above = anchor()
        .top()
        .as_top()
        .adjust(-(ROW_H + space::LG))
        .with(ROW_H.px().as_height());
    let line = g.forest.branch(
        hero,
        // Seeded at its final character count, and left with no glyph colours: the first tick
        // writes both. `glyph_colors` on the builder fixes a map at spawn, which cannot express
        // a highlight that moves.
        Text::new(hud_line(0.0, 0.0, 0.0))
            // The line is a fixed 39 characters, so its width is purely a function of the
            // size: at `LABEL` that is ~280px, which is wider than a short *and* narrow
            // window has to give. It is the one element here that cannot wrap or truncate
            // gracefully -- the breakpoint scale only means anything whole.
            .size(FontSize::new(type_scale::LABEL).short(HUD_SHORT))
            .color(role::on_surface_variant())
            // On `short` it goes to the very top of the screen instead of above the wordmark.
            // Landscape puts the wordmark in the left column and the buttons in the right, and
            // a line this wide centred over both would run straight through them -- but the
            // strip above both columns is empty, and the line is the one piece of the hero
            // that needs no room made for it.
            .at(Location::new()
                .xs(0.pct().as_left().with(100.pct().as_right()), above)
                .short(
                    0.pct().as_left().with(100.pct().as_right()),
                    space::XL.px().as_top().with(ROW_H.px().as_height()),
                ))
            .elevate(Elevation::up(3))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            .anchored(wordmark)
            .opacity(0.0),
    );
    fade_in(g.forest, line, seq, AT_READOUT);
    g.page.readout = Some(Readout {
        leaf: line,
        ms: 0.0,
        since: 0.0,
        shown: None,
        lit: None,
    });
}

/// The chevron's placement, `drift` px lower than its resting spot.
///
/// Built in one place because the bob animates a whole `Location`: a target carrying only
/// `xs` would drop the `short` variant, snapping the chevron back to the stacked position
/// the moment the loop first ran on a landscape viewport.
fn chevron_at(drift: i32) -> Location {
    let bottom = |gap: i32| {
        100.pct()
            .as_bottom()
            .adjust(-(CHEVRON + gap) + drift)
            .with(CHEVRON.px().as_height())
    };
    Location::new()
        .xs(
            50.pct().as_center_x().with(CHEVRON.px().as_width()),
            bottom(space::MD),
        )
        .short(
            (SHORT_TEXT_RIGHT_PCT / 2.0)
                .pct()
                .as_center_x()
                .with(CHEVRON.px().as_width()),
            bottom(space::SM),
        )
}

/// "more", with a chevron breathing under it at the fold.
///
/// Both are one control. The word and the chevron sit in a contiguous band -- the label's box
/// ends exactly where the chevron's begins -- so a single target spanning the pair is the same
/// reach the label already claimed, minus the dead gap between them and the dead margin either
/// side of a centred chevron. It is also static while the chevron bobs, so the thing you are
/// aiming at holds still.
fn hint(g: &mut Grow, hero: Leaf, seq: Leaf) {
    // Both the word and the chevron are placed by their *bottom* edge, so the band has to be
    // derived the same way rather than guessed from the chevron's inset: the word's top is a
    // label-height above its own gap, and the chevron's bottom is a bob lower than its resting
    // one. Written from the wrong end, the band sat below the word entirely and hung into the
    // empty strip under the chevron.
    //
    // Top and bottom, not bottom twice -- both edges are measured from the foot of the hero,
    // but a pair has to name two different edges or the resolver has nothing to solve.
    let band = |label_gap: i32, chevron_gap: i32| {
        100.pct()
            .as_top()
            .adjust(-(CHEVRON + label_gap + HINT_H))
            .with(
                100.pct()
                    .as_bottom()
                    .adjust(-(CHEVRON + chevron_gap) + BOB_PX),
            )
    };
    let target = g.forest.branch(
        hero,
        Bare::new()
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    band(space::XL, space::MD),
                )
                .short(
                    0.pct()
                        .as_left()
                        .with(SHORT_TEXT_RIGHT_PCT.pct().as_right()),
                    band(space::LG, space::SM),
                ))
            .elevate(Elevation::up(2))
            .interactive(),
    );
    // armed on the label's fade, the earlier of the two -- once the word is legible the
    // control means something, and the chevron joining it 120ms later does not change that
    crate::site::arm_at(g, target, AT_HINT + crate::site::motion::FADE);
    // the hero's own next stop, and the whole reason the hint is a control at all
    g.page.nav.push((target, 1));

    let label = g.forest.branch(
        hero,
        Text::new(HINT_TEXT)
            .size(FontSize::new(type_scale::TITLE))
            .color(role::on_surface_variant())
            .at(Location::new()
                .xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    100.pct()
                        .as_bottom()
                        .adjust(-(CHEVRON + space::XL))
                        .with(HINT_H.px().as_height()),
                )
                // stays under the text column, clear of the buttons on the right
                .short(
                    0.pct()
                        .as_left()
                        .with(SHORT_TEXT_RIGHT_PCT.pct().as_right()),
                    100.pct()
                        .as_bottom()
                        .adjust(-(CHEVRON + space::LG))
                        .with(HINT_H.px().as_height()),
                ))
            .elevate(Elevation::up(3))
            .align(HorizontalAlignment::Center, VerticalAlignment::Middle)
            // both pass through to the band behind them, or each would win the hit-test on its
            // own pixels and split the one control back into two
            .pass_through()
            .opacity(0.0),
    );
    fade_in(g.forest, label, seq, AT_HINT);

    let chevron = g.forest.branch(
        hero,
        Icon::new(IconId::from(IconHandles::ChevronDown))
            .color(role::accent())
            .at(chevron_at(0))
            .elevate(Elevation::up(3))
            .pass_through()
            .opacity(0.0),
    );
    fade_in(g.forest, chevron, seq, AT_HINT + 120);

    // Off the entrance sequence and running forever, backtracking so it returns rather than
    // snapping -- a plain loop would jerk back to the top every cycle. Tied to the chevron's
    // own lifetime, so leaving the route stops it.
    g.forest.animate(
        chevron,
        Motion::Location(chevron_at(BOB_PX)),
        timing(AT_HINT + 200, BOB_MS, Ease::DECELERATE)
            .repeat(Repeat::Forever)
            .backtrack(),
    );
}
