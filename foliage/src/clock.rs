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
    pub(crate) fn advance(&mut self, millis: u64) {
        self.pending += Duration::from_millis(millis);
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
