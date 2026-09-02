use core::time::Duration;

/// The frame's one source of time.
///
/// Sampled once, at the top of the frame; every phase after that reads the same instant.
pub(crate) struct Clock {
    elapsed: Duration,
    delta: Duration,
    pending: Duration,
}

impl Clock {
    pub(crate) fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
            delta: Duration::ZERO,
            pending: Duration::ZERO,
        }
    }

    /// Fixes the instant this frame runs at.
    pub(crate) fn sample(&mut self) {
        self.delta = core::mem::take(&mut self.pending);
        self.elapsed += self.delta;
    }

    /// Moves the clock forward, to be taken up by the next [`sample`](Clock::sample).
    ///
    /// The one way time enters the engine. The platform loop advances it by what actually elapsed
    /// between frames and the headless suite advances it by hand, so both reach [`sample`] the same
    /// way and a frame cannot tell which it is running under.
    ///
    /// [`sample`]: Clock::sample
    pub(crate) fn advance(&mut self, delta: Duration) {
        self.pending += delta;
    }

    /// Time since the engine was built.
    pub(crate) fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// How long the last frame took.
    pub(crate) fn delta(&self) -> Duration {
        self.delta
    }
}
