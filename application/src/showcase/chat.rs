//! A chat with a model: the conversation, the reply as it arrives, and what steers it.

use foliage::{
    Boxed, Corners, Elevation, Font, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Pollen, Rounding, Source, Text, Timing, Tween, content, left, right, top,
};
use lichen::{Chip, Pick, Press, Say, Voice, gate, measure};

use crate::icons::Icons;
use crate::parts::{self, Stack, at, in_lane, lane};

/// What the model says, word by word.
const REPLY: &str = "Start with a pothos. It takes low light, it forgives a missed week of \
                     water, and it says when it is thirsty by drooping -- then stands back up an \
                     hour after a drink.";

/// How long one word takes to arrive.
const WORD_MS: u64 = 55;

/// How many tokens the context holds, how many it starts with, and how many one attachment adds.
const WINDOW: u32 = 8_000;
const START: u32 = 2_400;
const ATTACH: u32 = 1_300;

/// The steps a tool-using answer goes through.
const TOOLS: [&str; 3] = ["search", "read", "answer"];

/// The conversations on the side, the first one open.
const THREADS: [&str; 3] = [
    "a first houseplant",
    "refactor the parser",
    "sourdough ratios",
];

pub(crate) struct Chat {
    regenerate: Chip,
    stop: Chip,
    reply: Leaf,
    streamed: usize,
    ticking: Option<Tween>,
    ask: lichen::Field,
    send: Chip,
    sent: Say,
    model: Pick,
    tools: Vec<Chip>,
    called: [bool; TOOLS.len()],
    answered: Say,
    attach: Chip,
    meter: Leaf,
    used: Leaf,
    tokens: u32,
    threads: Vec<Chip>,
    open: usize,
    prompt: lichen::Field,
}

impl Chat {
    pub(crate) fn grow(grove: &mut Grove, page: Leaf, icons: &Icons, italic: Font) -> Self {
        let m = measure();
        let mut stack = Stack::new(page);
        stack.heading(
            grove,
            "llm chat",
            "A conversation with a model, and the few things a person steers one with.",
            italic,
        );

        // Two turns: the person's on the right, raised; the model's on the left, sunk into the
        // card, so whose is whose is read before a word of either.
        let body = stack.card(grove, "conversation", 164.0);
        bubble(
            grove,
            body,
            "what's a good first plant for a dark flat?",
            (left(30.pct()).right(100.pct()), 0.0, 58.0),
            Palette::Raised,
        );
        bubble(
            grove,
            body,
            "A zz plant or a pothos. Both take low light, and both would rather be forgotten \
             than fussed over.",
            (left(0.px()).right(82.pct()), 66.0, 98.0),
            Palette::Surface,
        );

        // A reply arriving a word at a time, and the two presses that start one and stop it.
        let body = stack.card(grove, "streaming reply", lane(1) + 100.0);
        let regenerate = parts::chip(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("regenerate")), 0),
            icons.refresh_cw,
            "regenerate",
        );
        let stop = parts::chip(
            grove,
            body,
            in_lane(right(100.pct()).width(Chip::width("stop")), 0),
            icons.square,
            "stop",
        );
        let reply = parts::words(
            grove,
            body,
            "",
            Location::new().xs(
                left(0.px()).width(100.pct()),
                top(lane(1).px()).height(content()),
            ),
            lichen::REST.ink,
        );

        // Asking: a line to type in, and a send that is only the press to make once there is
        // something to send.
        let body = stack.card(grove, "composer", lane(1) + 16.0);
        let ask = lichen::Field::grow(
            grove,
            body,
            in_lane(left(0.px()).right(100.pct() - (m.height + m.gap).px()), 0),
            4.0,
            "ask",
            false,
        );
        let mut send = parts::mark(
            grove,
            body,
            in_lane(right(100.pct()).width(m.height.px()), 0),
            icons.send,
        );
        send.arm(grove, Press::Inert);
        let sent = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        let body = stack.card(grove, "model", m.height);
        let model = Pick::grow(
            grove,
            body,
            Location::new(),
            6.0,
            "model",
            &["small", "medium", "large"],
            1,
        );

        // An answer that uses tools, one step at a time: each is pressed as it runs.
        let body = stack.card(grove, "tool calls", lane(1) + 16.0);
        let marks = [icons.search, icons.file_text, icons.zap];
        let tools = TOOLS
            .iter()
            .zip(marks)
            .enumerate()
            .map(|(n, (&name, mark))| {
                // A third each, less their share of the two gaps between them.
                let third = 100.0 / TOOLS.len() as f32;
                let across = left((n as f32 * third).pct() + (n as f32 * m.gap / 3.0).px())
                    .width(third.pct() - (2.0 * m.gap / 3.0).px());
                parts::chip(grove, body, in_lane(across, 0), mark, name)
            })
            .collect();
        let answered = Say::grow(
            grove,
            body,
            at(left(0.px()).width(100.pct()), lane(1), 16.0),
            Some(italic),
        );

        // How much of the context is spent: a press that adds to it, and a meter that says what
        // is left, turning to the caution hue as it fills.
        let body = stack.card(grove, "context", lane(1) + 8.0);
        let attach = parts::chip(
            grove,
            body,
            in_lane(left(0.px()).width(Chip::width("attach")), 0),
            icons.paperclip,
            "attach",
        );
        let used = parts::words(
            grove,
            body,
            "",
            Location::new().xs(
                right(100.pct()).width(content()),
                top(((m.height - 16.0) / 2.0).px()).height(1.letters()),
            ),
            lichen::REST.ink,
        );
        let track = grove.branch(
            body,
            Panel::new()
                .color(Palette::Surface)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(1))
                .at(at(left(0.px()).width(100.pct()), lane(1), 8.0)),
        );
        let meter = grove.branch(
            track,
            Panel::new()
                .color(Palette::Accent)
                .rounding(Corners::all(Rounding::Full))
                .intangible()
                .elevate(Elevation::up(1))
                .at(filled(START)),
        );

        // The conversations kept on the side: one open, the rest sunk.
        let body = stack.card(grove, "conversations", lane(THREADS.len()) - m.gap);
        let threads = THREADS
            .iter()
            .enumerate()
            .map(|(n, &name)| {
                parts::chip(
                    grove,
                    body,
                    in_lane(left(0.px()).width(100.pct()), n),
                    icons.message_square,
                    name,
                )
            })
            .collect();

        // What the model is told before anything is asked: put there rather than typed, so it
        // reads in the grey between until the person makes it theirs.
        let body = stack.card(grove, "system prompt", 76.0);
        let mut prompt = lichen::Field::grow(grove, body, Location::new(), 6.0, "system", true);
        prompt.put(
            grove,
            "Be careful and brief. Say so when you are not sure, and never guess a number.",
        );

        let mut chat = Self {
            regenerate,
            stop,
            reply,
            streamed: 0,
            ticking: None,
            ask,
            send,
            sent,
            model,
            tools,
            called: [false; TOOLS.len()],
            answered,
            attach,
            meter,
            used,
            tokens: START,
            threads,
            open: 0,
            prompt,
        };
        chat.stream(grove);
        chat.call(grove);
        chat.spend(grove);
        chat.thread(grove);
        chat
    }

    pub(crate) fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if self.regenerate.pressed(pollen) {
            self.stream(grove);
        }
        if self.stop.pressed(pollen)
            && let Some(ticking) = self.ticking.take()
        {
            grove.stop(ticking);
            self.streaming(grove);
        }
        if let Some(ticking) = self.ticking
            && pollen.finished(ticking)
        {
            self.word(grove);
        }

        if self.ask.frame(grove, pollen) {
            let typed = !self.ask.value(grove).trim().is_empty();
            self.send.arm(grove, Press::armed_if(typed));
        }
        if (self.send.pressed(pollen) || pollen.submitted(self.ask.input()))
            && !self.ask.value(grove).trim().is_empty()
        {
            self.sent.tell(
                grove,
                &format!("sent to the {} model.", self.model.chosen()),
                Voice::Hint,
            );
            self.ask.clear(grove);
            self.send.arm(grove, Press::Inert);
        }
        self.model.frame(grove, pollen);

        if let Some(n) = self.tools.iter().position(|chip| chip.pressed(pollen)) {
            match self.called.iter().all(|&done| done) {
                true => self.called = [false; TOOLS.len()],
                false => self.called[n] = true,
            }
            self.call(grove);
        }

        if self.attach.pressed(pollen) {
            self.tokens = (self.tokens + ATTACH).min(WINDOW);
            self.spend(grove);
        }

        if let Some(n) = self.threads.iter().position(|chip| chip.pressed(pollen))
            && n != self.open
        {
            self.open = n;
            self.thread(grove);
        }
        self.prompt.frame(grove, pollen);
    }

    /// Starts the reply again from its first word.
    fn stream(&mut self, grove: &mut Grove) {
        if let Some(ticking) = self.ticking.take() {
            grove.stop(ticking);
        }
        self.streamed = 0;
        grove.text(self.reply, "");
        self.ticking = Some(grove.timer(Timing::ms(WORD_MS)));
        self.streaming(grove);
    }

    /// One more word, and the timer for the next -- or nothing, if that was the last.
    fn word(&mut self, grove: &mut Grove) {
        let words: Vec<&str> = REPLY.split_whitespace().collect();
        self.streamed = (self.streamed + 1).min(words.len());
        grove.text(self.reply, words[..self.streamed].join(" "));
        self.ticking = match self.streamed < words.len() {
            true => Some(grove.timer(Timing::ms(WORD_MS))),
            false => None,
        };
        if self.ticking.is_none() {
            self.streaming(grove);
        }
    }

    /// Dresses the two presses for whether a reply is arriving: stop is the one to make while it
    /// is, and has nothing to stop while it is not.
    fn streaming(&mut self, grove: &mut Grove) {
        let arriving = self.ticking.is_some();
        self.stop.arm(grove, Press::rest_if(arriving));
        self.regenerate.arm(grove, Press::Rest);
    }

    /// Dresses the tool steps for which have run, and says what came of them once all have.
    fn call(&mut self, grove: &mut Grove) {
        let all = self.called.iter().all(|&done| done);
        for (n, (chip, press)) in self.tools.iter_mut().zip(gate(&self.called)).enumerate() {
            let press = match (all, n == TOOLS.len() - 1) {
                (true, true) => Press::Chosen,
                _ => press,
            };
            chip.arm(grove, press);
        }
        match all {
            true => self
                .answered
                .tell(grove, "answered from three sources.", Voice::Hint),
            false => self.answered.set(grove, "press each step as it runs."),
        }
    }

    /// Writes how much of the context is spent, and fills the meter to it.
    fn spend(&mut self, grove: &mut Grove) {
        let full = self.tokens * 100 / WINDOW;
        grove.text(
            self.used,
            format!("{} / {} tokens", thousands(self.tokens), thousands(WINDOW)),
        );
        let fill = match full {
            0..80 => Palette::Accent,
            80..100 => Palette::Caution,
            _ => Palette::Danger,
        };
        grove.animate(
            self.meter,
            Motion::Location(filled(self.tokens)),
            lichen::timing(),
        );
        grove.animate(self.meter, fill.into(), lichen::timing());
        self.attach.arm(grove, Press::rest_if(self.tokens < WINDOW));
    }

    /// Dresses the conversations for which is open: it chosen, the rest wells.
    fn thread(&mut self, grove: &mut Grove) {
        for (n, chip) in self.threads.iter_mut().enumerate() {
            match n == self.open {
                true => chip.arm(grove, Press::Chosen),
                false => chip.wear(grove, lichen::WELL),
            }
        }
    }
}

/// A turn of the conversation: a rounded ground in `fill`, and the words wrapped inside it.
fn bubble(
    grove: &mut Grove,
    under: Leaf,
    says: &str,
    (across, top_px, height): (foliage::Horizontal, f32, f32),
    fill: Palette,
) {
    let ground = grove.branch(
        under,
        Panel::new()
            .color(fill)
            .rounding(Corners::all(Rounding::Md))
            .intangible()
            .elevate(Elevation::up(1))
            .at(at(across, top_px, height)),
    );
    grove.branch(
        ground,
        Text::new(says)
            .color(Palette::Ink)
            .font_size(lichen::caption())
            .intangible()
            .elevate(Elevation::up(1))
            .at(Location::new().xs(
                left(12.px()).right(100.pct() - 12.px()),
                top(10.px()).height(content()),
            )),
    );
}

/// The meter's fill at `tokens` of the window.
fn filled(tokens: u32) -> Location {
    let share = tokens as f32 / WINDOW as f32 * 100.0;
    Location::new().xs(
        left(0.px()).width(share.pct()),
        top(0.px()).bottom(100.pct()),
    )
}

/// A count, with its thousands set apart.
fn thousands(count: u32) -> String {
    match count >= 1_000 {
        true => format!("{},{:03}", count / 1_000, count % 1_000),
        false => format!("{count}"),
    }
}
