/// The scalar channels one animated value decomposes into, in a fixed order.
///
/// [`Animate`](crate::Animate) implementations build this in `interpolations` -- a
/// `Color` contributes four channels, a `Section` four, an `Opacity` one -- and read the
/// tweened numbers back by the same index in `apply`. The order is the contract between
/// those two halves; nothing else enforces it.
#[derive(Clone, Default)]
pub struct Interpolations {
    pub(crate) scalars: Vec<Interpolation>,
}

impl Interpolations {
    /// An empty set; add channels with [`with`](Self::with).
    pub fn new() -> Self {
        Self { scalars: vec![] }
    }
    /// Flips every channel end-for-end, so the next pass runs back the way it came --
    /// what [`Animation::backtrack`](crate::Animation::backtrack) does between passes.
    pub(crate) fn reverse(&mut self) {
        for scalar in self.scalars.iter_mut() {
            std::mem::swap(&mut scalar.start, &mut scalar.finish);
            scalar.diff = -scalar.diff;
        }
    }
    /// Appends a channel running from `s` to `e`. Its index is its position in the chain.
    pub fn with(mut self, s: f32, e: f32) -> Self {
        self.scalars.push(Interpolation::new(s, e));
        self
    }
    /// Takes channel `i`'s current value, or `None` if it has not advanced this frame.
    /// Consuming, so each tick's value is applied once.
    pub fn read(&mut self, i: usize) -> Option<f32> {
        self.scalars.get_mut(i)?.current_value()
    }
    /// Channel `i`'s progress as a fraction of its total span, for values that need the
    /// proportion rather than the number.
    pub fn read_percent(&mut self, i: usize) -> Option<f32> {
        self.scalars.get_mut(i)?.percent()
    }
}

/// One scalar channel: where it starts, where it ends, and its value this frame.
#[derive(Copy, Clone)]
pub struct Interpolation {
    pub(crate) start: f32,
    pub(crate) finish: f32,
    pub(crate) diff: f32,
    pub(crate) current_value: Option<f32>,
}

impl Interpolation {
    /// A channel from `s` to `e`. `diff` is signed, so a decreasing channel carries a
    /// negative span.
    pub fn new(s: f32, e: f32) -> Self {
        Self {
            start: s,
            finish: e,
            diff: e - s,
            current_value: None,
        }
    }
    /// Takes this frame's value, leaving `None` behind.
    pub fn current_value(&mut self) -> Option<f32> {
        self.current_value.take()
    }
    /// This frame's value as a fraction of the channel's own span.
    pub fn percent(&self) -> Option<f32> {
        self.current_value.and_then(|v| Option::from(v / self.diff))
    }
}
