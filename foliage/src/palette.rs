//! Palette -- the tones a fill is stated in, and the scheme that answers them.

use bevy_ecs::component::Component;
use tracing::trace;

use crate::color::Color;

/// How many neutral roles a [`Scheme`] answers: the three grounds, and the ink read on them.
const NEUTRALS: usize = 4;

/// How many hues are named: accent, signal, danger, caution, positive.
const NAMED: usize = 5;

/// How many hues a [`Scheme`] answers, the named ones and the slots.
const HUES: usize = NAMED + Palette::SLOTS;

/// How many forms a hue is held in: the ground, the mark, and what is read on the ground.
const FORMS: usize = 3;

/// How many steps a ramp holds.
const STEPS: usize = 5;

/// How many tones a [`Scheme`] holds: a ramp for each neutral, and a ramp for each form of each hue.
const TONES: usize = NEUTRALS * STEPS + HUES * FORMS * STEPS;

/// How far one step moves a role's seed, in OKLab lightness.
///
/// Large enough that a step reads as a different color against the one beside it, small enough that
/// two steps do not read as a different role.
const NOTCH: f32 = 0.06;

/// Where a mark derived from a ground is set in OKLab lightness: against a dark ground, and against a
/// light one. Text lightness, so what is derived reads on the surfaces the way ink does.
const MARK: (f32, f32) = (0.84, 0.44);

/// Where what is read on a ground is set in OKLab lightness, if it is derived: on a bright ground,
/// and on a deep one.
const ON: (f32, f32) = (0.19, 0.97);

/// The OKLab lightness above which a ground is bright, and what is read on it is deep.
const BRIGHT: f32 = 0.62;

/// How much of its ground's chroma a derived `on` keeps: a tint of the hue, so the pair reads as one
/// thing, and not a color of its own.
const TINT: f32 = 0.15;

/// What a color is for, rather than what it is.
///
/// An element declares a tone and the [`Scheme`] decides what it resolves to, so a treatment is
/// stated once and every element carrying that tone follows when it changes. A literal has no way to
/// be changed together with the others, which is why one cannot be written here.
///
/// A tone is a role, a form, and a step on that form's ramp. The role is what the color is for and
/// is what a scheme is written in terms of; the step is how far the tone sits from the ground the
/// scheme is read against, which is what a state -- a hover, a press, something disabled -- is
/// expressed as without leaving the scheme.
///
/// # Neutrals, and hues
///
/// Four roles are neutral: three grounds -- [`Surface`](Palette::Surface),
/// [`Raised`](Palette::Raised), [`Muted`](Palette::Muted) -- and the [`Ink`](Palette::Ink) read
/// against all three. The rest are hues, and a hue is held in three forms, because one color cannot
/// do all three jobs a hue is given:
///
/// - its **ground**, what is filled with it -- the role as named, `Palette::Accent`;
/// - its **mark**, what a glyph, a rule or a run of text is set in to carry the hue against the
///   neutral grounds -- [`mark`](Palette::mark);
/// - and what is **on** it: read against the ground form -- [`on`](Palette::on).
///
/// The three are different lightnesses of one hue and no step connects them: a ramp reaches two
/// notches either side of its seed, and the distance from a fill to something legible on it is
/// several times that. So each form is its own ramp, seeded or derived from the ground, and a tone
/// never has to leave the scheme to say "this hue, as a mark".
///
/// Five hues are named, for the jobs nearly every app has -- [`Accent`](Palette::Accent),
/// [`Signal`](Palette::Signal), [`Danger`](Palette::Danger), [`Caution`](Palette::Caution),
/// [`Positive`](Palette::Positive) -- and [`SLOTS`](Palette::SLOTS) more are numbered, for whatever
/// an app wants to say that those do not. An app names a slot for itself: `const MOSS: Palette =
/// Palette::hue(0);`.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Palette {
    role: Role,
    form: Form,
    step: Step,
}

// The roles, each named at its base step. Cased as the roles they name rather than as the constants
// they are, because a tone is what a caller writes and `Palette::Accent` is its name.
#[allow(non_upper_case_globals)]
impl Palette {
    /// The ordinary fill, and what an element that says nothing takes.
    pub const Surface: Self = Self::base(Role::Surface);
    /// A surface in front of another: a card against the page it sits on.
    pub const Raised: Self = Self::base(Role::Raised);
    /// A quieter fill, for a division or a rule.
    pub const Muted: Self = Self::base(Role::Muted);
    /// What is read against a surface rather than drawn as one.
    pub const Ink: Self = Self::base(Role::Ink);
    /// The emphatic hue, and what the app's own content is marked with.
    pub const Accent: Self = Self::base(Role::Accent);
    /// The second hue: what the system says, as against what the app is.
    ///
    /// A separate role and not a step of [`Accent`](Palette::Accent), because the two answer
    /// different questions and a scheme is expected to move them independently.
    pub const Signal: Self = Self::base(Role::Signal);
    /// The hue of what cannot be taken back, and of what went wrong.
    pub const Danger: Self = Self::base(Role::Danger);
    /// The hue of what wants care before it is done: a warning, something about to lapse.
    pub const Caution: Self = Self::base(Role::Caution);
    /// The hue of what went right, or was chosen.
    pub const Positive: Self = Self::base(Role::Positive);
    /// What is read against [`Accent`](Palette::Accent): its [`on`](Palette::on) form.
    ///
    /// Kept by this name because it was a role of its own before every hue had one.
    pub const Contrast: Self = Self::Accent.on();

    /// How many numbered hues there are beside the named ones.
    pub const SLOTS: usize = 4;

    /// Numbered hue `n`, at its ground form, for an app to name for itself.
    ///
    /// A slot no scheme has seeded answers as [`Accent`](Palette::Accent) does, form for form and
    /// step for step, so a slot that was forgotten is plainly the accent rather than nothing.
    ///
    /// # Panics
    ///
    /// If `n` is not below [`SLOTS`](Palette::SLOTS).
    pub const fn hue(n: usize) -> Self {
        assert!(
            n < Self::SLOTS,
            "a hue slot is numbered below Palette::SLOTS"
        );
        Self::base(Role::Slot(n as u8))
    }
}

impl Palette {
    /// This role at its base step, in its ground form.
    const fn base(role: Role) -> Self {
        Self {
            role,
            form: Form::Ground,
            step: Step::Base,
        }
    }

    /// The same role at `step`.
    pub const fn at(self, step: Step) -> Self {
        Self {
            role: self.role,
            form: self.form,
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

    /// This hue as a fill, at the same step. The form a hue is named in.
    ///
    /// A neutral is a ground already, except [`Ink`](Palette::Ink), whose ground is
    /// [`Surface`](Palette::Surface).
    pub const fn ground(self) -> Self {
        match self.role {
            Role::Ink => Palette::Surface.at(self.step),
            _ => self.form(Form::Ground),
        }
    }

    /// This hue as a mark, at the same step: what carries it on the neutral grounds -- a glyph, a
    /// rule, a word set apart.
    ///
    /// A neutral's mark is [`Ink`](Palette::Ink).
    pub const fn mark(self) -> Self {
        match self.role {
            Role::Surface | Role::Raised | Role::Muted | Role::Ink => Palette::Ink.at(self.step),
            _ => self.form(Form::Mark),
        }
    }

    /// What is read on this hue's ground, at the same step.
    ///
    /// On a neutral ground that is [`Ink`](Palette::Ink); on ink, it is
    /// [`Surface`](Palette::Surface), which is what ink is read against.
    pub const fn on(self) -> Self {
        match self.role {
            Role::Surface | Role::Raised | Role::Muted => Palette::Ink.at(self.step),
            Role::Ink => Palette::Surface.at(self.step),
            _ => self.form(Form::On),
        }
    }

    /// The same hue and step in `form`.
    const fn form(self, form: Form) -> Self {
        Self {
            role: self.role,
            form,
            step: self.step,
        }
    }

    /// The same tone with its ramp's farthest step, which is where the ramp starts in a
    /// [`Scheme`].
    const fn ramp(self) -> Self {
        self.at(Step::Farthest)
    }

    /// Where a [`Scheme`] holds this tone's color.
    fn index(self) -> usize {
        match self.role.hue() {
            None => self.role.neutral() * STEPS + self.step.index(),
            Some(hue) => {
                NEUTRALS * STEPS + (hue * FORMS + self.form.index()) * STEPS + self.step.index()
            }
        }
    }
}

/// What a color is for.
///
/// Not public: a role is reached as the tone at its base step -- [`Palette::Accent`] -- and moved
/// along its ramp and between its forms from there, so there is one name for a role rather than
/// two.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum Role {
    #[default]
    Surface,
    Raised,
    Muted,
    Ink,
    Accent,
    Signal,
    Danger,
    Caution,
    Positive,
    Slot(u8),
}

impl Role {
    /// Which hue this is, counting the named ones first, or `None` for a neutral.
    const fn hue(self) -> Option<usize> {
        match self {
            Role::Surface | Role::Raised | Role::Muted | Role::Ink => None,
            Role::Accent => Some(0),
            Role::Signal => Some(1),
            Role::Danger => Some(2),
            Role::Caution => Some(3),
            Role::Positive => Some(4),
            Role::Slot(n) => Some(NAMED + n as usize),
        }
    }

    /// Which neutral this is. Only asked of a neutral.
    fn neutral(self) -> usize {
        match self {
            Role::Surface => 0,
            Role::Raised => 1,
            Role::Muted => 2,
            Role::Ink => 3,
            _ => unreachable!("a hue is not a neutral"),
        }
    }
}

/// Which of a hue's three forms a tone is. See [`Palette`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
enum Form {
    #[default]
    Ground,
    Mark,
    On,
}

impl Form {
    /// Where among a hue's ramps this form's is.
    const fn index(self) -> usize {
        match self {
            Form::Ground => 0,
            Form::Mark => 1,
            Form::On => 2,
        }
    }
}

/// How far a tone stands from the ground the scheme it resolves under is read against.
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
    /// A tone, resolved against whatever [`Scheme`] is in force when it is drawn.
    Role(Palette),
    /// A color stated outright, which no repaint moves.
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
/// A ramp of five steps for each neutral and for each form of each hue, written at boot or at any
/// frame after it with [`repaint`](crate::Grow::repaint). Changing it changes every element carrying
/// an affected tone and nothing else: extraction resolves the tone each frame and compares the
/// result, so the elements that moved are exactly the ones that were painted in a color that
/// changed.
///
/// A ramp is stated in one color, its seed, and derives its other four steps from it -- so a theme
/// is a handful of decisions rather than every tone. Derivation holds the seed's hue and chroma and
/// moves only its lightness, in OKLab, away from the ground the reading names; a step that leaves
/// sRGB has its chroma backed off until it fits. A single step can be replaced outright where a
/// derived one will not do.
///
/// A hue goes further: seeding its ground derives its mark and what is read on it as well, unless
/// either was seeded outright. The mark is the ground's hue at text lightness; what is on it is
/// near-black or near-white, whichever the ground is not, with a tint of the hue.
///
/// # Grounds, and the marks read against them
///
/// A ramp reaches two notches either side of its seed, so **every seed is either a ground or a
/// mark, and no step moves one into the other**: the distance from a fill to something legible on
/// that fill is several times what a ramp spans. That is arithmetic rather than convention, and it
/// is the whole reason the tones come in the pairs they do.
///
/// | Ground | Read against it |
/// |---|---|
/// | [`Surface`](Palette::Surface), [`Raised`](Palette::Raised), [`Muted`](Palette::Muted) | [`Ink`](Palette::Ink), or any hue's [`mark`](Palette::mark) |
/// | any hue | that hue's [`on`](Palette::on) |
///
/// Nothing enforces this. A tone is an index into a table of colors, and an element filled with
/// [`Ink`](Palette::Ink) or lettered in [`Muted`](Palette::Muted) draws exactly as asked. What the
/// forms carry is which seeds were chosen as partners -- [`on`](Palette::on) names the partner of
/// anything -- and the failure they prevent is silent, because an illegible tone still renders.
///
/// # Spectra
///
/// Beside the tones, a scheme holds [`SPECTRA`](Scheme::SPECTRA) numbered runs of colors -- a
/// gradient's stops -- for whatever colors many things along a line rather than one thing in one
/// tone. Nothing in foliage reads them; they are here so that the colors an app draws from are all
/// stated in one place, and so a repaint moves them with the rest. See
/// [`spectrum`](Scheme::spectrum).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scheme {
    tones: [Color; TONES],
    reading: Reading,
    /// Which forms of which hues were seeded outright rather than derived from their ground: one
    /// bit per form of each hue.
    seeded: u64,
    spectra: [[Color; Scheme::STOPS]; Scheme::SPECTRA],
    /// How many stops of each spectrum are stated. None is unstated.
    stops: [u8; Scheme::SPECTRA],
}

impl Scheme {
    /// How many numbered spectra a scheme holds.
    pub const SPECTRA: usize = 4;

    /// How many stops a spectrum can hold.
    pub const STOPS: usize = 8;

    /// The scheme every tone resolves to until one is given: a dark neutral ground with a green
    /// accent.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same roles read against a light ground.
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
                Color::rgb(0.10, 0.11, 0.13),
            ],
            [
                Color::rgb(0.20, 0.52, 0.34),
                Color::rgb(0.19, 0.42, 0.60),
                Color::rgb(0.72, 0.20, 0.16),
                Color::rgb(0.70, 0.50, 0.08),
                Color::rgb(0.22, 0.52, 0.26),
            ],
            Reading::Light,
        )
    }

    /// What one tone resolves to.
    ///
    /// A ramp's base step is its seed: writing it states the ramp and re-derives the rest of it --
    /// and, for a hue's ground, re-derives its mark and what is on it, where those were not seeded
    /// outright themselves. Writing any other step replaces that step alone and leaves the ramp it
    /// sits in as it was, which is the way out when a derived step is not the one wanted; a later
    /// seed of the same ramp derives over it.
    pub fn set(mut self, tone: Palette, color: Color) -> Self {
        if tone.step != Step::Base {
            self.tones[tone.index()] = color;
            return self;
        }
        self.write(tone, color);
        if let Some(hue) = tone.role.hue() {
            self.seeded |= seeded(hue, tone.form);
            if tone.form == Form::Ground {
                for form in [Form::Mark, Form::On] {
                    if self.seeded & seeded(hue, form) == 0 {
                        self.write(tone.form(form), derive(color, form, self.reading));
                    }
                }
            }
        }
        self
    }

    /// The color `tone` resolves to.
    pub fn color(&self, tone: Palette) -> Color {
        let tone = match tone.role {
            Role::Slot(_) if !self.stated(tone.role) => Palette {
                role: Role::Accent,
                ..tone
            },
            _ => tone,
        };
        self.tones[tone.index()]
    }

    /// Spectrum `n`, stated as its stops, first to last.
    ///
    /// # Panics
    ///
    /// If `n` is not below [`SPECTRA`](Scheme::SPECTRA), or `stops` is not two to
    /// [`STOPS`](Scheme::STOPS) colors.
    pub fn spectrum(mut self, n: usize, stops: &[Color]) -> Self {
        assert!(
            n < Self::SPECTRA,
            "a spectrum is numbered below Scheme::SPECTRA"
        );
        assert!(
            (2..=Self::STOPS).contains(&stops.len()),
            "a spectrum is two to Scheme::STOPS stops"
        );
        self.spectra[n][..stops.len()].copy_from_slice(stops);
        self.stops[n] = stops.len() as u8;
        self
    }

    /// The stops of spectrum `n`, first to last.
    ///
    /// A spectrum no one has stated is the accent's ramp, from its farthest step to its nearest, so
    /// a forgotten one is plainly the accent rather than nothing.
    ///
    /// # Panics
    ///
    /// If `n` is not below [`SPECTRA`](Scheme::SPECTRA).
    pub fn stops(&self, n: usize) -> Vec<Color> {
        assert!(
            n < Self::SPECTRA,
            "a spectrum is numbered below Scheme::SPECTRA"
        );
        match self.stops[n] {
            0 => Step::ALL
                .into_iter()
                .map(|step| self.color(Palette::Accent.at(step)))
                .collect(),
            stated => self.spectra[n][..stated as usize].to_vec(),
        }
    }

    /// Neutral and named-hue seeds and a reading, as the ramps they derive.
    fn seeded(neutrals: [Color; NEUTRALS], hues: [Color; NAMED], reading: Reading) -> Self {
        let mut scheme = Self {
            tones: [Color::rgb(0.0, 0.0, 0.0); TONES],
            reading,
            seeded: 0,
            spectra: [[Color::rgb(0.0, 0.0, 0.0); Scheme::STOPS]; Scheme::SPECTRA],
            stops: [0; Scheme::SPECTRA],
        };
        let named = [
            Palette::Accent,
            Palette::Signal,
            Palette::Danger,
            Palette::Caution,
            Palette::Positive,
        ];
        for (tone, seed) in [
            Palette::Surface,
            Palette::Raised,
            Palette::Muted,
            Palette::Ink,
        ]
        .into_iter()
        .zip(neutrals)
        {
            scheme.write(tone, seed);
        }
        for (tone, seed) in named.into_iter().zip(hues) {
            for form in [Form::Ground, Form::Mark, Form::On] {
                scheme.write(tone.form(form), derive(seed, form, reading));
            }
        }
        // The slots answer as the accent until they are seeded, which `color` does by reading the
        // accent's ramps in their place; what is held for them is never read until then.
        scheme
    }

    /// `seed`'s ramp, written where `tone`'s is held.
    fn write(&mut self, tone: Palette, seed: Color) {
        let at = tone.ramp().index();
        self.tones[at..at + STEPS].copy_from_slice(&ramp(seed, self.reading));
    }

    /// Whether any form of `role` has been seeded.
    fn stated(&self, role: Role) -> bool {
        let hue = role.hue().expect("only a hue is seeded");
        (0..FORMS).any(|form| self.seeded & (1 << (hue * FORMS + form)) != 0)
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
                Color::rgb(0.93, 0.94, 0.96),
            ],
            [
                Color::rgb(0.38, 0.71, 0.51),
                Color::rgb(0.36, 0.63, 0.82),
                Color::rgb(0.80, 0.30, 0.26),
                Color::rgb(0.88, 0.68, 0.26),
                Color::rgb(0.46, 0.72, 0.36),
            ],
            Reading::Dark,
        )
    }
}

/// The bit that says `form` of hue `hue` was seeded outright.
fn seeded(hue: usize, form: Form) -> u64 {
    1 << (hue * FORMS + form.index())
}

/// The seed of `form`, derived from a hue's ground.
///
/// A mark is the ground's hue and chroma at the text lightness of the reading, so it reads on the
/// neutral grounds the way ink does. What is on a ground is near-black on a bright one and
/// near-white on a deep one, keeping a tint of the hue. Either backs its chroma off to fit sRGB.
fn derive(ground: Color, form: Form, reading: Reading) -> Color {
    let (lightness, a, b) = ground.oklab();
    let (to, keep) = match form {
        Form::Ground => return ground,
        Form::Mark => (
            match reading {
                Reading::Dark => MARK.0,
                Reading::Light => MARK.1,
            },
            1.0,
        ),
        Form::On => (if lightness > BRIGHT { ON.0 } else { ON.1 }, TINT),
    };
    Color::from_oklab(to, a * keep, b * keep, ground.alpha).0
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
