//! The studio: a visitor's own words, grown by the model, through a queue that is not there.
//!
//! The whole of the flow is the mock's: the request is accepted or refused by the gallery's `api`
//! the moment it is made, and the [`Courier`] hands the answer back late -- *accepted*, then the
//! queue moving up a place at a time -- before the picture is painted and developed here.

use std::time::Duration;

use foliage::{
    Boxed, Elevation, Fit, Font, Grove, Grow, Image, Leaf, Location, Palette, Place, Plate, Pollen,
    Sap, Source, Stem, Text, Vein, anchor, aspect, content, left, right, top,
};
use lichen::{Chip, Field, Pick, Press, Say, Voice, measure};

use arboretum::genart::{Painter, Params, Seed, Style};
use arboretum::parts::ping;
use arboretum::sprout::{Arrival, Courier, Emerge, Progress};

use crate::api::{self, Job, Refusal};
use crate::develop::{Develop, Event, area, blank};
use crate::frame::PICTURE;
use crate::icons::Icons;
use crate::theme::{self, column};

const STYLES: &[&str] = &["bloom", "flow", "lattice", "orbit", "moire", "caustic", "aurora", "lava"];
const STEPS: &[&str] = &["20", "35", "50"];

/// How long the queue takes to accept, and to move up a place.
const ACCEPT_MS: u64 = 700;
const MOVE_MS: u64 = 650;
/// How long a step of sampling is shown for.
const STEP_MS: u64 = 60;

/// A picture a visitor grew, for the wall.
pub struct Grown {
    pub prompt: String,
    pub picture: Vec<u8>,
}

/// What comes back from the queue.
enum Answer {
    Accepted(Result<Job, Refusal>),
    Moved(Job, u32),
}

enum Stage {
    Idle,
    Waiting,
    Sampling(Job, Develop, Duration),
    Grown(Job),
}

pub struct Studio {
    root: Leaf,
    prompt: Field,
    style: Pick,
    steps: Pick,
    grow: Chip,
    suggest: Chip,
    copy: Chip,
    say: Say,
    plate: Plate,
    progress: Progress,
    readout: Leaf,
    ping: lichen::Ping,
    courier: Courier<Answer>,
    stage: Stage,
    asked: u32,
    tail: Leaf,
    arriving: Vec<Arrival>,
    shown: bool,
}

impl Studio {
    pub fn grow(grove: &mut Grove, page: Leaf, after: Leaf, face: Font, icons: &Icons) -> Self {
        let m = measure();
        let root = grove.branch(
            page,
            Stem::new()
                .intangible()
                .anchored(after)
                .font_size(lichen::caption())
                .at(Location::new().xs(column(), top(anchor().bottom() + 120.px()).height(10.px()))),
        );
        let heading = grove.branch(
            root,
            Text::new("studio")
                .font(face)
                .font_size(theme::heading())
                .color(Palette::Ink)
                .intangible()
                .opacity(0.0)
                .at(Location::new().xs(left(0.px()).width(content()), top(0.px()).height(content()))),
        );
        let blurb = grove.branch(
            root,
            Text::new("grow your own. the model is shared with every visitor, so there is a queue, and it has opinions")
                .font_size(lichen::caption())
                .color(Palette::Muted)
                .intangible()
                .opacity(0.0)
                .anchored(heading)
                .at(Location::new().xs(
                    left(0.px()).right(100.pct()).at_most(560.px()),
                    top(anchor().bottom() + 10.px()).height(content()),
                )),
        );
        let pick = Pick::height(2);
        let form_height = m.height + m.gap + pick + m.gap + Pick::height(1) + m.gap * 2.0 + m.height + m.gap + m.height;
        let form = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(blurb)
                .opacity(0.0)
                .font_size(lichen::caption())
                .at(Location::new()
                    .xs(left(0.px()).right(100.pct()), top(anchor().bottom() + 28.px()).height(form_height.px()))
                    .lg(left(0.px()).width(52.pct()), top(anchor().bottom() + 32.px()).height(form_height.px()))),
        );
        let row = |y: f32, h: f32| Location::new().xs(left(0.px()).right(100.pct()), top(y.px()).height(h.px()));
        let mut y = 0.0;
        let prompt = Field::grow(grove, form, row(y, m.height), 7.0, "prompt", false);
        prompt.hint(grove, "moss on a sleeping piano");
        y += m.height + m.gap;
        let style = Pick::grow(grove, form, row(y, pick), 7.0, "style", STYLES, 2);
        y += pick + m.gap;
        let steps = Pick::grow(grove, form, row(y, Pick::height(1)), 7.0, "steps", STEPS, 1);
        y += Pick::height(1) + m.gap * 2.0;
        let chips = grove.branch(
            form,
            Stem::new().intangible().font_size(lichen::caption()).at(row(y, m.height)),
        );
        let at = |from: foliage::Length, name: &str| {
            Location::new().xs(left(from).width(Chip::width(name)), top(0.px()).height(100.pct()))
        };
        let mut grow = Chip::grow(grove, chips, at(0.px(), "grow"), Elevation::up(1), icons.zap, "grow");
        grow.arm(grove, Press::Armed);
        let after_grow = Chip::width("grow") + m.gap.px();
        let suggest = Chip::grow(grove, chips, at(after_grow.clone(), "suggest"), Elevation::up(1), icons.shuffle, "suggest");
        let after_suggest = after_grow + Chip::width("suggest") + m.gap.px();
        let mut copy = Chip::grow(grove, chips, at(after_suggest, "copy seed"), Elevation::up(1), icons.copy, "copy seed");
        copy.arm(grove, Press::Inert);
        y += m.height + m.gap;
        let say = Say::grow(grove, form, row(y, m.height), None);
        // The result: a dark plate until there is something to develop, and the queue's progress
        // under it.
        let plate = grove.plate();
        grove.load(plate, blank((PICTURE, PICTURE)), area((PICTURE, PICTURE)));
        let picture = grove.branch(
            root,
            Image::new(plate)
                .fit(Fit::Crop)
                .intangible()
                .opacity(0.0)
                .anchored(form)
                .at(Location::new()
                    .xs(
                        left(0.px()).right(100.pct()).at_most(380.px()),
                        top(anchor().bottom() + 28.px()).height(aspect(1.0)),
                    )
                    .lg(
                        left(58.pct()).right(100.pct()).at_most(400.px()),
                        top(anchor().top()).height(aspect(1.0)),
                    )),
        );
        let under = grove.branch(
            root,
            Stem::new()
                .intangible()
                .anchored(picture)
                .opacity(0.0)
                .font_size(lichen::caption())
                .at(Location::new().xs(
                    left(anchor().left()).right(anchor().right()),
                    top(anchor().bottom() + 14.px()).height(44.px()),
                )),
        );
        let progress = Progress::grow(
            grove,
            under,
            Location::new().xs(left(0.px()).right(100.pct()), top(0.px()).height(4.px())),
            1,
            Palette::Raised,
            Palette::Accent,
        );
        let readout = grove.branch(
            under,
            Text::new("the plate is waiting for words")
                .color(Palette::Muted)
                .intangible()
                .at(Location::new().xs(left(0.px()).right(100.pct() - Chip::square() - 8.px()), top(16.px()).height(content()))),
        );
        // A tick beside the readout when something lands: grown, or a seed copied.
        let ping = ping(
            grove,
            under,
            Location::new().xs(right(100.pct()).width(Chip::square()), top(10.px()).height(m.height.px())),
            icons.check,
        );
        Self {
            root,
            prompt,
            style,
            steps,
            grow,
            suggest,
            copy,
            say,
            plate,
            progress,
            readout,
            ping,
            courier: Courier::new(0x57d1_0000),
            stage: Stage::Idle,
            asked: 0,
            tail: under,
            arriving: vec![
                Arrival::words(heading, Some(face), theme::heading(), 7.0),
                Arrival::words(blurb, None, lichen::caption(), 3.0),
                form.into(),
                picture.into(),
                under.into(),
            ],
            shown: false,
        }
    }

    pub fn leaf(&self) -> Leaf {
        self.root
    }

    /// The last thing in the studio, which the wall stands below.
    pub fn tail(&self) -> Leaf {
        self.tail
    }

    /// Carries the studio. Hands back a picture the frame it is grown.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen, emerge: &mut Emerge) -> Option<Grown> {
        if !self.shown
            && let Some(Sap::Section(drawn)) = grove.tap(self.root, Vein::Drawn)
            && drawn.top() < grove.viewport().height * 0.86
        {
            self.shown = true;
            emerge.bring(grove, &self.arriving, 0, 120);
        }
        self.prompt.frame(grove, pollen);
        self.style.frame(grove, pollen);
        self.steps.frame(grove, pollen);
        self.ping.frame(grove, pollen);
        if self.suggest.pressed(pollen) {
            self.asked += 1;
            // Imported, not typed: it reads in the grey between until the visitor makes it theirs.
            self.prompt.put(grove, api::suggest(self.asked));
        }
        if self.copy.pressed(pollen)
            && let Stage::Grown(job) = &self.stage
        {
            grove.copy(format!("0x{:016x}", job.seed));
            self.ping.fire(grove);
            self.say.tell(grove, "seed copied: the same words and style grow it again", Voice::Hint);
        }
        let busy = matches!(self.stage, Stage::Waiting | Stage::Sampling(..));
        if (self.grow.pressed(pollen) || pollen.submitted(self.prompt.input())) && !busy {
            self.request(grove);
        }
        for (_, answer) in self.courier.frame(pollen) {
            match answer {
                Answer::Accepted(Ok(job)) => {
                    let place = job.ahead + 1;
                    self.say.tell(grove, &format!("accepted · queued at position {place}"), Voice::Quiet);
                    grove.text(self.readout, format!("queued · position {place}"));
                    self.courier.send(grove, MOVE_MS, Answer::Moved(job, place - 1));
                }
                Answer::Accepted(Err(refusal)) => {
                    self.stage = Stage::Idle;
                    self.grow.arm(grove, Press::Armed);
                    self.say.tell(grove, refusal.words(), Voice::Warn);
                    grove.text(self.readout, "refused · nothing was grown");
                    self.progress.set(grove, 0.0, 200);
                }
                Answer::Moved(job, 0) => self.sample(grove, job),
                Answer::Moved(job, place) => {
                    grove.text(self.readout, format!("queued · position {place}"));
                    self.progress.set(grove, 0.08 / place as f32, 300);
                    self.courier.send(grove, MOVE_MS, Answer::Moved(job, place - 1));
                }
            }
        }
        let mut grown = None;
        if let Stage::Sampling(job, develop, began) = &mut self.stage {
            match develop.frame(grove) {
                Some(Event::Step(step, of)) => {
                    grove.text(self.readout, format!("sampling · step {step}/{of}"));
                    self.progress.set(grove, 0.1 + 0.9 * step as f32 / of as f32, STEP_MS * 2);
                }
                Some(Event::Developed) => {
                    let took = grove.elapsed().saturating_sub(*began).as_secs_f32();
                    let job = job.clone();
                    grove.text(self.readout, format!("grown in {took:.1}s · seed 0x{:04x} · {} steps", job.seed & 0xffff, job.steps));
                    self.progress.set(grove, 1.0, 200);
                    self.say.tell(grove, "grown. it is on the visitors' wall now", Voice::Hint);
                    self.ping.fire(grove);
                    grown = Some(Grown {
                        prompt: job.prompt.clone(),
                        picture: develop.picture().map(<[u8]>::to_vec).unwrap_or_default(),
                    });
                    self.grow.arm(grove, Press::Armed);
                    self.copy.arm(grove, Press::Rest);
                    self.stage = Stage::Grown(job);
                }
                Some(Event::Painted) => {
                    grove.text(self.readout, "sampling · step 0");
                }
                None => {}
            }
        }
        grown
    }

    fn request(&mut self, grove: &mut Grove) {
        let words = self.prompt.value(grove);
        let style = Style::ALL[self.style.index()];
        let steps: u32 = STEPS[self.steps.index()].parse().unwrap_or(35);
        let answer = api::generate(&words, style, steps, self.asked + 2);
        // Nothing to send is said at once: there is no reason to ask a server about it.
        if answer == Err(Refusal::Empty) {
            self.say.tell(grove, Refusal::Empty.words(), Voice::Warn);
            self.prompt.focus(grove);
            return;
        }
        self.asked += 1;
        self.stage = Stage::Waiting;
        self.grow.arm(grove, Press::Inert);
        self.copy.arm(grove, Press::Inert);
        self.progress.set(grove, 0.02, 200);
        self.say.tell(grove, "sending your words to the model…", Voice::Quiet);
        grove.text(self.readout, "connecting");
        self.courier.send(grove, ACCEPT_MS, Answer::Accepted(answer));
    }

    fn sample(&mut self, grove: &mut Grove, job: Job) {
        let ramp = theme::ramp_for(&job.prompt);
        let seed = Seed::new(job.seed);
        let painter = Painter::new(job.style, Params::of(job.style, seed), &ramp, PICTURE, PICTURE, 0.0);
        let develop = Develop::new(grove, self.plate, painter, job.steps, job.steps as u64 * STEP_MS, seed);
        self.say.tell(grove, "sampling…", Voice::Quiet);
        grove.text(self.readout, "sampling · painting the latent");
        self.stage = Stage::Sampling(job, develop, grove.elapsed());
    }
}
