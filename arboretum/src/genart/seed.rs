//! Where every drawing's numbers come from.

/// A deterministic stream of numbers from one seed: SplitMix64.
///
/// Its own generator rather than the platform's, for the reason lichen's `Scatter` is: a piece is
/// the same piece on every run and every target, so a seed printed on a placard is a seed someone
/// can grow again. Unlike `Scatter` it can be seeded, which is the whole of the difference.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    /// The stream that starts from `seed`.
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The stream a piece of text names -- a prompt, a product's name -- by FNV-1a over its bytes,
    /// with letter case and surrounding space set aside so that "Moss" and " moss" are one prompt.
    pub fn of(text: &str) -> Self {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in text.trim().bytes() {
            hash ^= byte.to_ascii_lowercase() as u64;
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        Self(hash)
    }

    /// What this stream started from, as printed on a placard.
    pub const fn value(self) -> u64 {
        self.0
    }

    /// A second, independent stream derived from this one and `n`, without drawing from it.
    ///
    /// What a piece's parts take, so adding a part to a drawing does not move the numbers every
    /// other part was drawn with.
    pub fn fork(self, n: u64) -> Self {
        let mut forked = Self(self.0 ^ n.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        forked.next_u64();
        forked
    }

    /// The next 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// The next number in `0.0..1.0`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// The next number between two bounds.
    pub fn between(&mut self, low: f32, high: f32) -> f32 {
        low + self.unit() * (high - low)
    }

    /// The next whole number below `bound`.
    pub fn below(&mut self, bound: u32) -> u32 {
        (((self.next_u64() >> 32) * bound as u64) >> 32) as u32
    }

    /// The low 32 bits, for the noise functions, which are keyed by one.
    pub fn key(self) -> u32 {
        (self.0 ^ (self.0 >> 32)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::Seed;

    #[test]
    fn a_seed_is_the_same_stream_every_time() {
        let mut one = Seed::new(7);
        let mut two = Seed::new(7);
        for _ in 0..64 {
            assert_eq!(one.next_u64(), two.next_u64());
        }
    }

    #[test]
    fn numbers_stay_in_range() {
        let mut seed = Seed::new(0x5eed);
        for _ in 0..10_000 {
            let n = seed.unit();
            assert!((0.0..1.0).contains(&n));
            assert!(seed.below(12) < 12);
        }
    }

    #[test]
    fn text_names_a_seed_regardless_of_case_and_edges() {
        assert_eq!(Seed::of("Moss Liturgy"), Seed::of("  moss liturgy "));
        assert_ne!(Seed::of("moss"), Seed::of("fern"));
    }

    #[test]
    fn forks_are_independent_of_each_other() {
        let base = Seed::new(1);
        assert_ne!(base.fork(1), base.fork(2));
        assert_eq!(base.fork(3), base.fork(3));
    }
}
