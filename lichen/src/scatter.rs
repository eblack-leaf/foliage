//! The jitter everything is built from.

/// A deterministic sequence of numbers, from one fixed seed.
///
/// Its own generator rather than the platform's, so a drawing is the same drawing on every run and
/// on every target: a mosaic that came out differently each time would be a different mosaic. The
/// caller holds one and hands it to every `grow`, so two things grown in a row are two drawings
/// rather than copies.
pub struct Scatter(u64);

impl Scatter {
    /// The one sequence, from a fixed seed.
    pub fn new() -> Self {
        Self(0x9E37_79B9_7F4A_7C15)
    }

    /// The next number in `0.0..1.0`.
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1 << 24) as f32
    }

    /// The next number between two bounds.
    pub fn between(&mut self, low: f32, high: f32) -> f32 {
        low + self.next() * (high - low)
    }
}

impl Default for Scatter {
    fn default() -> Self {
        Self::new()
    }
}
