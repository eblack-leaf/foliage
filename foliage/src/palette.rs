//! Palette -- the tones a fill is stated in, and the scheme that answers them.

use bevy_ecs::component::Component;
use tracing::trace;

use crate::color::Color;

/// How many roles a [`Scheme`] answers.
const ROLES: usize = 6;

/// How many steps a role's ramp holds.
const STEPS: usize = 5;

/// How far one step moves a role's seed, in OKLab lightness.
///
/// Large enough that a step reads as a different color against the one beside it, small enough that
/// two steps do not read as a different role.
const NOTCH: f32 = 0.06;

/// What a color is for, rather than what it is.
///
/// An element declares a tone and the [`Scheme`] decides what it resolves to, so a treatment is
/// stated once and every element carrying that tone follows when it changes. A literal has no way to
/// be changed together with the others, which is why one cannot be written here.
///
/// A tone is a role and a step on that role's ramp. The role is what the color is for and is what a
/// scheme is written in terms of; the step is how far the tone sits from the ground the scheme is
/// read against, which is what a state -- a hover, a press, something disabled -- is expressed as
/// without leaving the scheme. Each of the six roles names its own base step, so `Palette::Accent`
/// is a tone in its own right and every element that declares one says nothing about steps.
///
/// Resolution happens in extraction, against the tone the element declared. Nothing on the element
/// holds a resolved color, so there is no copy to fall out of date and repainting is one write.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Palette {
    role: Role,
    step: Step,
}

// The six roles, each named at its base step. Cased as the roles they name rather than as the
// constants they are, because a tone is what a caller writes and `Palette::Accent` is its name.
#[allow(non_upper_case_globals)]
impl Palette {
    /// The ordinary fill, and what an element that says nothing takes.
    pub const Surface: Self = Self::base(Role::Surface);
    /// A surface in front of another: a card against the page it sits on.
    pub const Raised: Self = Self::base(Role::Raised);
    /// A quieter fill, for a division or a rule.
    pub const Muted: Self = Self::base(Role::Muted);
    /// The one emphatic color.
    pub const Accent: Self = Self::base(Role::Accent);
    /// What is read against a surface rather than drawn as one.
    pub const Ink: Self = Self::base(Role::Ink);
    /// What is read against [`Accent`](Palette::Accent) rather than against a surface.
    ///
    /// The accent is the one role whose color a scheme is expected to move furthest, and a label on
    /// top of it cannot follow with [`Ink`](Palette::Ink) -- ink is legible against a surface by
    /// construction and against an accent only by accident. Stating both is what lets an accent be
    /// rehued without silently making everything written on it unreadable.
    pub const Contrast: Self = Self::base(Role::Contrast);
}

impl Palette {
    /// This role at its base step.
    const fn base(role: Role) -> Self {
        Self {
            role,
            step: Step::Base,
        }
    }

    /// The same role at `step`.
    pub const fn at(self, step: Step) -> Self {
        Self {
            role: self.role,
            step,
        }
    }

    /// The same role one step farther into the ground, and itself at [`Step::Farthest`].
    ///
    /// What something quieted -- disabled, placeholding, resting under something else -- is drawn
    /// in.
    pub const fn recede(self) -> Self {
        self.at(match self.step {
            Step::Farthest | Step::Far => Step::Farthest,
            Step::Base => Step::Far,
            Step::Near => Step::Base,
            Step::Nearest => Step::Near,
        })
    }

    /// The same role one step nearer out of the ground, and itself at [`Step::Nearest`].
    ///
    /// What something brought forward -- hovered, held, carrying focus -- is drawn in.
    pub const fn advance(self) -> Self {
        self.at(match self.step {
            Step::Farthest => Step::Far,
            Step::Far => Step::Base,
            Step::Base => Step::Near,
            Step::Near | Step::Nearest => Step::Nearest,
        })
    }

    /// Where a [`Scheme`] holds this tone's color.
    fn index(self) -> usize {
        self.role.index() * STEPS + self.step.index()
    }
}

/// What a color is for.
///
/// Not public: a role is reached as the tone at its base step -- [`Palette::Accent`] -- and moved
/// along its ramp from there, so there is one name for a role rather than two.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum Role {
    #[default]
    Surface,
    Raised,
    Muted,
    Accent,
    Ink,
    Contrast,
}

impl Role {
    /// Which ramp in a [`Scheme`] is this role's.
    fn index(self) -> usize {
        match self {
            Role::Surface => 0,
            Role::Raised => 1,
            Role::Muted => 2,
            Role::Accent => 3,
            Role::Ink => 4,
            Role::Contrast => 5,
        }
    }
}

/// How far a tone stands from the ground its scheme is read against.
///
/// Named for where a step stands rather than for which way it moves in lightness, because the two
/// readings a scheme has move opposite ways: standing nearer means lighter against a dark ground and
/// darker against a light one. A state written once as [`advance`](Palette::advance) is therefore
/// correct in both, which is what makes a light and a dark scheme the same app rather than two.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Step {
    /// Two steps back into the ground.
    Farthest,
    /// One step back into the ground.
    Far,
    /// What the role resolves to undeclared, and the color a [`Scheme`] states the role in.
    #[default]
    Base,
    /// One step out of the ground.
    Near,
    /// Two steps out of the ground.
    Nearest,
}

impl Step {
    /// Every step, deepest into the ground first, which is the order a ramp is held in.
    const ALL: [Step; STEPS] = [
        Step::Farthest,
        Step::Far,
        Step::Base,
        Step::Near,
        Step::Nearest,
    ];

    /// Where in a role's ramp this step sits.
    fn index(self) -> usize {
        match self {
            Step::Farthest => 0,
            Step::Far => 1,
            Step::Base => 2,
            Step::Near => 3,
            Step::Nearest => 4,
        }
    }

    /// How many notches out from the ground this step stands, relative to the base.
    fn notches(self) -> f32 {
        self.index() as f32 - Step::Base.index() as f32
    }
}

/// What an element is filled with: a tone, or a color stated outright.
///
/// A tone is the ordinary answer and the reason [`Palette`] exists -- a treatment stated once, and
/// moved for every element carrying it by one [`repaint`](crate::Grow::repaint).
///
/// A literal is the opt-out, and it is deliberately visible as one. An element filled with a color
/// is not part of any scheme, so a repaint does not move it; that is the whole of the difference,
/// and holding both in one type is what keeps it a difference a reader can see rather than two
/// parallel paths through the renderer. It is also what lets a fill be animated to either -- a
/// motion writes the target to the element, and a target has to be something the element can hold.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Fill {
    Role(Palette),
    Literal(Color),
}

impl Fill {
    /// The color this resolves to under `scheme`. A literal resolves to itself.
    pub(crate) fn color(self, scheme: &Scheme) -> Color {
        match self {
            Fill::Role(tone) => scheme.color(tone),
            Fill::Literal(color) => color,
        }
    }
}

impl Default for Fill {
    /// The ordinary fill's tone, which is what an element that says nothing takes.
    fn default() -> Self {
        Self::Role(Palette::default())
    }
}

impl From<Palette> for Fill {
    fn from(tone: Palette) -> Self {
        Self::Role(tone)
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Self::Literal(color)
    }
}

/// Which way a scheme's ramps run.
///
/// Not public: a reading is chosen by which constructor built the scheme, and the tones it produced
/// are the whole of what anything downstream reads.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Reading {
    Dark,
    Light,
}

impl Reading {
    /// Which way in OKLab lightness a step out from the ground goes.
    fn advancing(self) -> f32 {
        match self {
            Reading::Dark => 1.0,
            Reading::Light => -1.0,
        }
    }
}

/// What each [`Palette`] tone resolves to.
///
/// Six roles of five steps each, written at boot or at any frame after it with
/// [`repaint`](crate::Grow::repaint). Changing it changes every element carrying an affected tone
/// and nothing else: extraction resolves the tone each frame and compares the result, so the
/// elements that moved are exactly the ones that were painted in a color that changed.
///
/// A scheme is stated in six colors, one per role, and derives the other four steps of each ramp
/// from that seed -- so a theme is six decisions rather than thirty. Derivation holds the seed's hue
/// and chroma and moves only its lightness, in OKLab, away from the ground the reading names; a step
/// that leaves sRGB has its chroma backed off until it fits. A single step can be replaced outright
/// where a derived one will not do.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scheme {
    tones: [Color; ROLES * STEPS],
    reading: Reading,
}

impl Scheme {
    /// The scheme every tone resolves to until one is given: a dark neutral ground with a green
    /// accent.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same six roles read against a light ground.
    ///
    /// A separate scheme rather than a flag on this one, because which colors a role is seeded with
    /// is a decision and not a transform: a green that carries an accent against near-black is not
    /// the green that carries it against near-white. What the reading decides is only which way the
    /// ramps run from those seeds, so a tone an app declared is correct in both without moving.
    pub fn light() -> Self {
        Self::seeded(
            [
                Color::rgb(0.96, 0.96, 0.97),
                Color::rgb(1.0, 1.0, 1.0),
                Color::rgb(0.66, 0.68, 0.72),
                Color::rgb(0.20, 0.52, 0.34),
                Color::rgb(0.10, 0.11, 0.13),
                Color::rgb(0.97, 0.98, 0.99),
            ],
            Reading::Light,
        )
    }

    /// What one tone resolves to.
    ///
    /// A role's base step is its seed: writing it states the role and re-derives the rest of its
    /// ramp. Writing any other step replaces that step alone and leaves the ramp it sits in as it
    /// was, which is the way out when a derived step is not the one wanted.
    pub fn set(mut self, tone: Palette, color: Color) -> Self {
        if tone.step == Step::Base {
            let at = tone.role.index() * STEPS;
            self.tones[at..at + STEPS].copy_from_slice(&ramp(color, self.reading));
        } else {
            self.tones[tone.index()] = color;
        }
        self
    }

    /// The color `tone` resolves to.
    pub fn color(&self, tone: Palette) -> Color {
        self.tones[tone.index()]
    }

    /// Six seeds and a reading, as the ramps they derive.
    fn seeded(seeds: [Color; ROLES], reading: Reading) -> Self {
        let mut tones = [Color::rgb(0.0, 0.0, 0.0); ROLES * STEPS];
        for (role, seed) in seeds.into_iter().enumerate() {
            let at = role * STEPS;
            tones[at..at + STEPS].copy_from_slice(&ramp(seed, reading));
        }
        Self { tones, reading }
    }

    /// How many tones differ between this scheme and `other`.
    ///
    /// What a repaint reports. Every element painted in one of them is re-extracted and no other is,
    /// so this is the size of the repaint rather than a description of it.
    pub(crate) fn moved(&self, other: &Self) -> usize {
        self.tones
            .iter()
            .zip(other.tones.iter())
            .filter(|(held, given)| held != given)
            .count()
    }
}

impl Default for Scheme {
    fn default() -> Self {
        Self::seeded(
            [
                Color::rgb(0.09, 0.10, 0.12),
                Color::rgb(0.15, 0.16, 0.19),
                Color::rgb(0.28, 0.30, 0.34),
                Color::rgb(0.38, 0.71, 0.51),
                Color::rgb(0.93, 0.94, 0.96),
                Color::rgb(0.07, 0.08, 0.09),
            ],
            Reading::Dark,
        )
    }
}

/// One role's five steps, derived from its seed.
///
/// The seed is written at [`Step::Base`] rather than derived at zero offset, so a color a scheme was
/// given is exactly the color that role resolves to and a round trip through OKLab cannot move it.
///
/// The two halves of a ramp are sized independently. A role seeded near black or near white has two
/// notches of room on one side and less than that on the other, and the short half is compressed to
/// what is left rather than run off the end, so a ramp is ordered from its farthest step to its
/// nearest whatever it was seeded with. What that costs is a smaller step on the short side,
/// which is the most a seed at an extreme can be given: an ink already at white has no brighter
/// reading to offer. A seed sitting exactly on black or white has no room at all on that side, and
/// every step of that half answers the seed.
fn ramp(seed: Color, reading: Reading) -> [Color; STEPS] {
    let (lightness, a, b) = seed.oklab();
    let advancing = reading.advancing();
    let ground = if advancing > 0.0 {
        lightness
    } else {
        1.0 - lightness
    };
    let (out, back) = (notch(1.0 - ground), notch(ground));
    let mut steps = [seed; STEPS];
    for step in Step::ALL {
        let notches = step.notches();
        let size = if notches > 0.0 { out } else { back };
        let moved = (lightness + advancing * notches * size).clamp(0.0, 1.0);
        // The base, and any step of a half with no room in it. Left as the seed rather than
        // converted back from a lightness it never left, which a round trip would move by a bit.
        if moved == lightness {
            continue;
        }
        let (color, refitted) = Color::from_oklab(moved, a, b, seed.alpha);
        if refitted {
            trace!(?step, "ramp step backed off to fit sRGB");
        }
        steps[step.index()] = color;
    }
    steps
}

/// How far one step moves, given the lightness left between the seed and that end of the range.
///
/// A full notch where two of them fit, and half of what is there where they do not.
fn notch(headroom: f32) -> f32 {
    NOTCH.min(headroom / 2.0)
}
