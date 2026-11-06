//! A band of words running past, round and round.

use foliage::{
    Axes, Boxed, Ease, Elevation, Fill, Font, FontSize, Grove, Grow, Leaf, Location, Motion, Place,
    Pollen, Position, Sap, Source, Stem, Text, Timing, Tween, Vein, center_y, content, left,
};

/// How many characters one pass of the band runs to at least, so a band as wide as any window
/// never shows its end.
const PASS: usize = 300;

/// A band of words running leftward forever, as a ticker does.
///
/// One run of glyphs, saying the words over and over, clipped by a region that never scrolls and
/// drawn aside by a linear [`Motion::Shift`] of exactly one pass of the words -- after which the
/// run is in the same place it began, so the restart is invisible.
pub struct Marquee {
    band: Leaf,
    track: Leaf,
    passes: usize,
    speed: f32,
    span: Option<f32>,
    running: Option<Tween>,
    paused: bool,
}

impl Marquee {
    /// A band under `under`, standing where `at` says, saying `words` round and round at `size` in
    /// `font` and `fill`, `speed` pixels a second.
    #[allow(clippy::too_many_arguments)]
    pub fn grow(
        grove: &mut Grove,
        under: Leaf,
        at: Location,
        words: &str,
        size: FontSize,
        font: Option<Font>,
        fill: impl Into<Fill>,
        speed: f32,
    ) -> Self {
        let unit = words.chars().count().max(1);
        let passes = PASS.div_ceil(unit).max(1);
        let pass = words.repeat(passes);
        let band = grove.branch(
            under,
            Stem::new()
                .intangible()
                .scrolls(Axes::Horizontal)
                .font_size(size)
                .at(at),
        );
        let text = Text::new(format!("{pass}{pass}"))
            .color(fill)
            .font_size(size)
            .intangible()
            .pinned()
            .elevate(Elevation::up(1));
        let text = match font {
            Some(font) => text.font(font),
            None => text,
        };
        let track = grove.branch(
            band,
            text.at(Location::new().xs(
                left(0.px()).width(content()),
                center_y(50.pct()).height(1.letters()),
            )),
        );
        Self {
            band,
            track,
            passes,
            speed,
            span: None,
            running: None,
            paused: false,
        }
    }

    /// The band.
    pub fn leaf(&self) -> Leaf {
        self.band
    }

    /// Holds the band still where it is, or lets it run on from the start of a pass.
    pub fn pause(&mut self, grove: &mut Grove, paused: bool) {
        if self.paused == paused {
            return;
        }
        self.paused = paused;
        match paused {
            true => {
                if let Some(running) = self.running.take() {
                    grove.stop(running);
                }
            }
            false => self.run(grove),
        }
    }

    /// Measures the band once it is laid out, and keeps it running.
    pub fn frame(&mut self, grove: &mut Grove, pollen: &Pollen) {
        if self.span.is_none() {
            let Some(Sap::Section(drawn)) = grove.tap(self.track, Vein::Drawn) else {
                return;
            };
            if drawn.width() <= 0.0 {
                return;
            }
            self.span = Some(drawn.width() / 2.0);
            let _ = self.passes;
            if !self.paused {
                self.run(grove);
            }
            return;
        }
        if let Some(running) = self.running
            && pollen.finished(running)
        {
            self.run(grove);
        }
    }

    fn run(&mut self, grove: &mut Grove) {
        let Some(span) = self.span else {
            return;
        };
        let ms = (span / self.speed.max(1.0) * 1000.0) as u64;
        grove.shift(self.track, Position::default());
        self.running = Some(grove.animate(
            self.track,
            Motion::Shift(Position::new(-span, 0.0)),
            Timing::ms(ms.max(1)).ease(Ease::Linear),
        ));
    }
}
