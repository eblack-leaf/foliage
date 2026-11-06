//! Things arriving one after another.

use foliage::{Ease, Grove, Grow, Leaf, Motion, Position, Sequence, Timing};

/// Brings `leaves` in one after another: each fades up from nothing and eases back from `from`,
/// drawn aside, `step` ms after the one before, over `ms`. Handed back is the group they run in,
/// whose [`sequence_finished`](foliage::Pollen::sequence_finished) is the last of them landing.
///
/// Shift and opacity only, so nothing is laid out again however long the list.
pub fn stagger(grove: &mut Grove, leaves: &[Leaf], from: Position, step: u64, ms: u64) -> Sequence {
    let group = grove.sequence();
    for (n, &leaf) in leaves.iter().enumerate() {
        let timing = Timing::ms(ms)
            .after(n as u64 * step)
            .ease(Ease::Decelerate)
            .within(group);
        grove.opacity(leaf, 0.0);
        grove.shift(leaf, from);
        grove.animate(leaf, Motion::Opacity(1.0), timing);
        grove.animate(leaf, Motion::Shift(Position::default()), timing);
    }
    group
}

/// Takes `leaves` away together: each fades and drifts toward `to`, over `ms`, the last a little
/// before the first so the list folds up from its end.
pub fn recede(grove: &mut Grove, leaves: &[Leaf], to: Position, step: u64, ms: u64) -> Sequence {
    let group = grove.sequence();
    let count = leaves.len();
    for (n, &leaf) in leaves.iter().enumerate() {
        let timing = Timing::ms(ms)
            .after((count - 1 - n) as u64 * step)
            .ease(Ease::Accelerate)
            .within(group);
        grove.animate(leaf, Motion::Opacity(0.0), timing);
        grove.animate(leaf, Motion::Shift(to), timing);
    }
    group
}
