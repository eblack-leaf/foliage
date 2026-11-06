//! What each good looks like through its label's window: the palette itself, the shader running,
//! the face on a poster, the sound as a waveform.
//!
//! Each is grown into a box the caller placed and is asleep until it is woken -- by the pointer
//! resting on its label, or by being the good on show -- when it does what the good does: a
//! palette fans its swatches and warms, a shader runs, a face cycles its glyphs, a sound breathes.

use foliage::{
    Area, Boxed, Color, Corners, Ease, Elevation, Fit, FontSize, Grove, Grow, Image, Leaf,
    Location, Motion, Panel, Place, Plate, Position, Rounding, Source, Stem, Text, Timing,
    center_x, center_y, content, left, top,
};
use lichen::{Cluster, Scatter};

use arboretum::genart::{Painter, Params, Ramp, Seed, Style};

use crate::catalog::{Face, GOODS, Lch, Look, Wave, waveform};
use crate::theme::{Faces, oklch};

/// How large a good is shown, which decides how finely its picture is painted and how large its
/// glyphs are set.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Scale {
    /// A thumbnail: the bag's lines and the related goods.
    Mini,
    /// A label in the grid.
    Card,
    /// The good on show at the front.
    Hero,
    /// The good opened.
    Detail,
}

impl Scale {
    fn plate(self) -> u32 {
        match self {
            Scale::Mini => 72,
            Scale::Card => 200,
            Scale::Hero => 224,
            Scale::Detail => 256,
        }
    }

    /// How large a palette's puff is, in a window `width` wide: a puff is grown in pixels, so it
    /// is grown again when the window crosses into another width.
    fn cluster(self, width: f32) -> f32 {
        match (self, foliage::Layout::of(Area::new(width, 1.0))) {
            (Scale::Mini, _) => 34.0,
            (Scale::Card, foliage::Layout::Xs | foliage::Layout::Sm) => 70.0,
            (Scale::Card, foliage::Layout::Md) => 78.0,
            (Scale::Card, foliage::Layout::Lg) => 86.0,
            (Scale::Card, foliage::Layout::Xl) => 100.0,
            (Scale::Hero, _) => 150.0,
            (Scale::Detail, foliage::Layout::Xs | foliage::Layout::Sm) => 140.0,
            (Scale::Detail, _) => 200.0,
        }
    }

    fn glyphs(self) -> FontSize {
        match self {
            Scale::Mini => FontSize::new().xs(28),
            Scale::Card => FontSize::new().xs(54).xl(76),
            Scale::Hero => FontSize::new().xs(96).md(120),
            Scale::Detail => FontSize::new().xs(120).md(170),
        }
    }

    fn alphabet(self) -> FontSize {
        match self {
            Scale::Mini => FontSize::new().xs(8),
            Scale::Card => FontSize::new().xs(11),
            Scale::Hero => FontSize::new().xs(13),
            Scale::Detail => FontSize::new().xs(13).md(16),
        }
    }

    /// How far in from its window's edges a picture's parts stand.
    fn inset(self) -> f32 {
        match self {
            Scale::Mini => 4.0,
            Scale::Card => 10.0,
            Scale::Hero | Scale::Detail => 16.0,
        }
    }
}

/// The glyphs a specimen cycles through while it is awake.
const GLYPHS: [&str; 6] = ["Aa", "Gg", "Rr", "Qq", "&?", "Ee"];
/// How long a specimen holds a pair of glyphs, a waveform a breath, and a shader a frame.
const GLYPH_MS: f32 = 620.0;
const BREATH_MS: f32 = 380.0;
const SHADER_FPS: f32 = 14.0;
/// How many bars a waveform is.
pub const BARS: usize = 32;
/// The moment a shader is shown at while it sleeps.
const STILL: f32 = 2.0;

enum Kind {
    Palette {
        swatches: Vec<Leaf>,
        pad: Leaf,
        cluster: Cluster,
        colours: Vec<Color>,
        size: f32,
        warm: lichen::Ramp,
    },
    Shader {
        plate: Plate,
        style: Style,
        params: Params,
        ramp: Ramp,
        side: u32,
        painter: Option<Box<Painter>>,
        t: f32,
        clock: f32,
        painted: bool,
    },
    Specimen {
        glyphs: Leaf,
        shown: usize,
        clock: f32,
    },
    Sound {
        bars: Vec<Leaf>,
        seed: u64,
        wave: Wave,
        clock: f32,
        t: f32,
    },
}

/// One good's picture, grown into its window.
pub struct Art {
    root: Leaf,
    good: usize,
    scale: Scale,
    kind: Kind,
    awake: bool,
    /// How fast a shader runs, as a multiple of its own pace; zero holds it.
    pub speed: f32,
}

impl Art {
    /// Grows `good`'s picture under `under`, standing where `at` says, at `scale`.
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        good: usize,
        scale: Scale,
        faces: &Faces,
    ) -> Self {
        let g = &GOODS[good];
        let ground = match g.look {
            Look::Palette(swatches) => swatches[0],
            Look::Shader { ramp, .. } => ramp[0],
            Look::Specimen { paper, .. } => paper,
            Look::Sound { ground, .. } => ground,
        };
        let root = grove.branch(under, Panel::new().color(oklch(ground)).intangible().at(at));
        let inset = scale.inset();
        let kind = match g.look {
            Look::Palette(swatches) => {
                // The swatches along the foot, lightest to the right, and over them a puff of the
                // palette's colours.
                let others = &swatches[1..];
                let share = 100.0 / others.len() as f32;
                let strip = grove.branch(
                    root,
                    Stem::new()
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(
                            left(0.px()).right(100.pct()),
                            top(72.pct()).bottom(100.pct()),
                        )),
                );
                let swatches = others
                    .iter()
                    .enumerate()
                    .map(|(n, &swatch)| {
                        grove.branch(
                            strip,
                            Panel::new()
                                .color(oklch(swatch))
                                .intangible()
                                .at(Location::new().xs(
                                    left((n as f32 * share).pct()).width(share.pct() + 0.5.px()),
                                    top(0.px()).bottom(100.pct()),
                                )),
                        )
                    })
                    .collect();
                let colours: Vec<Color> = others.iter().map(|&s| oklch(s)).collect();
                let size = scale.cluster(grove.viewport().width);
                let (pad, cluster) = puff(grove, root, good, size, &colours);
                let bright: Vec<Color> = others.iter().rev().take(2).map(|&s| oklch(s)).collect();
                Kind::Palette {
                    swatches,
                    pad,
                    cluster,
                    colours,
                    size,
                    warm: lichen::Ramp::of(&bright),
                }
            }
            Look::Shader { style, ramp, seed } => {
                let side = scale.plate();
                let plate = grove.plate();
                grove.load(plate, flat(ramp[0], 2), Area::new(2.0, 2.0));
                grove.branch(
                    root,
                    Image::new(plate)
                        .fit(Fit::Crop)
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new()),
                );
                let ramp = Ramp::new(&ramp.iter().map(|&s| oklch(s)).collect::<Vec<_>>());
                let params = Params::of(style, Seed::new(seed));
                Kind::Shader {
                    plate,
                    style,
                    params,
                    painter: Some(Box::new(Painter::new(
                        style, params, &ramp, side, side, STILL,
                    ))),
                    ramp,
                    side,
                    t: STILL,
                    clock: 0.0,
                    painted: false,
                }
            }
            Look::Specimen { face, ink, .. } => {
                let font = match face {
                    Face::Xanh => Some(faces.roman),
                    Face::Major => Some(faces.major),
                    Face::Plex => Some(faces.plex),
                    Face::JetBrains => None,
                };
                let set = |text: Text| match font {
                    Some(font) => text.font(font),
                    None => text,
                };
                let glyphs = grove.branch(
                    root,
                    set(Text::new(GLYPHS[0]))
                        .font_size(scale.glyphs())
                        .color(oklch(ink))
                        .intangible()
                        .elevate(Elevation::up(1))
                        .at(Location::new().xs(
                            center_x(50.pct()).width(content()),
                            center_y(44.pct()).height(1.letters()),
                        )),
                );
                if scale != Scale::Mini {
                    grove.branch(
                        root,
                        Panel::new()
                            .color(oklch(ink))
                            .intangible()
                            .elevate(Elevation::up(1))
                            .at(Location::new().xs(
                                left(inset.px()).right(100.pct() - inset.px()),
                                top(80.pct()).height(1.px()),
                            )),
                    );
                    grove.branch(
                        root,
                        set(Text::new("abc xyz 0123"))
                            .font_size(scale.alphabet())
                            .color(oklch(ink))
                            .intangible()
                            .elevate(Elevation::up(1))
                            .at(Location::new().xs(
                                left(inset.px()).width(content()),
                                top(80.pct() + 6.px()).height(1.letters()),
                            )),
                    );
                }
                Kind::Specimen {
                    glyphs,
                    shown: 0,
                    clock: 0.0,
                }
            }
            Look::Sound {
                ink, seed, kind, ..
            } => {
                let heights = waveform(seed, kind, BARS, 0.0);
                let span = 100.0 - 2.0 * 9.0;
                let slot = span / BARS as f32;
                let bars = heights
                    .iter()
                    .enumerate()
                    .map(|(n, &h)| {
                        grove.branch(
                            root,
                            Panel::new()
                                .color(oklch(ink))
                                .rounding(Corners::all(Rounding::Full))
                                .intangible()
                                .elevate(Elevation::up(1))
                                .at(bar(n, slot, h)),
                        )
                    })
                    .collect();
                Kind::Sound {
                    bars,
                    seed,
                    wave: kind,
                    clock: 0.0,
                    t: 0.0,
                }
            }
        };
        Self {
            root,
            good,
            scale,
            kind,
            awake: false,
            speed: 1.0,
        }
    }

    /// Fits the picture to a window `width` wide: a palette's puff is grown again at the size that
    /// width calls for, if it is not that size already.
    pub fn fit(&mut self, grove: &mut Grove, width: f32) {
        let scale = self.scale;
        let good = self.good;
        let root = self.root;
        if let Kind::Palette {
            pad,
            cluster,
            colours,
            size,
            ..
        } = &mut self.kind
        {
            let wanted = scale.cluster(width);
            if (wanted - *size).abs() < 0.5 {
                return;
            }
            grove.prune(*pad);
            let (new_pad, new_cluster) = puff(grove, root, good, wanted, colours);
            *pad = new_pad;
            *cluster = new_cluster;
            *size = wanted;
            self.awake = false;
        }
    }

    /// The window.
    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// Wakes the picture, or puts it back to sleep.
    pub fn wake(&mut self, grove: &mut Grove, awake: bool) {
        if self.awake == awake {
            return;
        }
        self.awake = awake;
        match &mut self.kind {
            Kind::Palette {
                swatches,
                cluster,
                warm,
                ..
            } => {
                match awake {
                    true => cluster.tint(grove, warm),
                    false => cluster.restore(grove),
                }
                // The swatches fan up from the left, one after another, like a deck of chips.
                for (n, &swatch) in swatches.iter().enumerate() {
                    let lift = if awake { -(6.0 + 3.0 * n as f32) } else { 0.0 };
                    grove.animate(
                        swatch,
                        Motion::Shift(Position::new(0.0, lift)),
                        Timing::ms(300).after(n as u64 * 45).ease(Ease::Emphasis),
                    );
                }
            }
            Kind::Specimen {
                glyphs,
                shown,
                clock,
            } => {
                if !awake && *shown != 0 {
                    *shown = 0;
                    grove.text(*glyphs, GLYPHS[0]);
                }
                *clock = 0.0;
            }
            Kind::Sound {
                bars,
                seed,
                wave,
                t,
                ..
            } => {
                if !awake {
                    *t = 0.0;
                    let heights = waveform(*seed, *wave, BARS, 0.0);
                    settle(grove, bars, &heights, 420);
                }
            }
            Kind::Shader { .. } => {}
        }
    }

    /// Carries the picture for a frame: paints what is still to be painted, `budget` steps of it at
    /// most, and moves what is awake along.
    pub fn frame(&mut self, grove: &mut Grove, budget: &mut u32) {
        let dt = grove.frame_time().as_secs_f32() * 1000.0;
        let awake = self.awake;
        let speed = self.speed;
        match &mut self.kind {
            Kind::Shader {
                plate,
                style,
                params,
                ramp,
                side,
                painter,
                t,
                clock,
                painted,
            } => {
                if awake && speed > 0.0 {
                    *t += dt / 1000.0 * speed;
                }
                if let Some(running) = painter {
                    if *budget == 0 {
                        grove.again();
                        return;
                    }
                    *budget -= 1;
                    grove.again();
                    if running.step() {
                        let done = painter.take().expect("a painter that just finished");
                        grove.load(
                            *plate,
                            done.into_pixels(),
                            Area::new(*side as f32, *side as f32),
                        );
                        *painted = true;
                        *clock = 0.0;
                    }
                    return;
                }
                if awake && speed > 0.0 {
                    grove.again();
                    *clock += dt;
                    if *clock >= 1000.0 / SHADER_FPS {
                        *clock = 0.0;
                        *painter = Some(Box::new(Painter::new(
                            *style, *params, ramp, *side, *side, *t,
                        )));
                    }
                }
            }
            Kind::Specimen {
                glyphs,
                shown,
                clock,
            } => {
                if !awake {
                    return;
                }
                grove.again();
                *clock += dt;
                if *clock >= GLYPH_MS {
                    *clock = 0.0;
                    *shown = (*shown + 1) % GLYPHS.len();
                    grove.text(*glyphs, GLYPHS[*shown]);
                }
            }
            Kind::Sound {
                bars,
                seed,
                wave,
                clock,
                t,
            } => {
                if !awake {
                    return;
                }
                grove.again();
                *clock += dt;
                *t += dt / 1000.0;
                if *clock >= BREATH_MS {
                    *clock = 0.0;
                    let heights = waveform(*seed, *wave, BARS, *t * 3.0);
                    settle(grove, bars, &heights, BREATH_MS as u64);
                }
            }
            Kind::Palette { .. } => {}
        }
    }
}

/// Where bar `n` of a waveform stands, `slot` percent of the window apart, `h` of its height.
fn bar(n: usize, slot: f32, h: f32) -> Location {
    let x = 9.0 + slot * (n as f32 + 0.5);
    Location::new().xs(
        center_x(x.pct()).width((slot * 0.56).pct()),
        center_y(50.pct()).height((h * 64.0).pct()),
    )
}

/// Moves a waveform's bars to `heights` over `ms`.
fn settle(grove: &mut Grove, bars: &[Leaf], heights: &[f32], ms: u64) {
    let slot = (100.0 - 2.0 * 9.0) / BARS as f32;
    for (n, (&leaf, &h)) in bars.iter().zip(heights).enumerate() {
        grove.animate(
            leaf,
            Motion::Location(bar(n, slot, h)),
            Timing::ms(ms).ease(Ease::Decelerate),
        );
    }
}

/// A flat picture `side` square of one colour: what a shader's plate holds until it is painted.
fn flat(colour: Lch, side: u32) -> Vec<u8> {
    let rgba = arboretum::genart::bytes(oklch(colour));
    rgba.repeat((side * side) as usize)
}

/// A palette's puff of its own colours, `size` across, high in its window.
fn puff(
    grove: &mut Grove,
    root: Leaf,
    good: usize,
    size: f32,
    colours: &[Color],
) -> (Leaf, Cluster) {
    let pad = grove.branch(
        root,
        Stem::new()
            .intangible()
            .elevate(Elevation::up(1))
            .at(Location::new().xs(
                center_x(50.pct()).width(size.px()),
                center_y(36.pct()).height(size.px()),
            )),
    );
    let mut scatter = Scatter::new();
    // Draw a few numbers off first, so no two palettes' puffs are the same drawing.
    for _ in 0..(good * 7) {
        scatter.next();
    }
    let cluster = Cluster::grow(grove, pad, size, &lichen::Ramp::of(colours), &mut scatter);
    (pad, cluster)
}
