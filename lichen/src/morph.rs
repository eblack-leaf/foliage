use foliage::{Animation, EcsExtension, Ease, Entity, Opacity, Polygon, Tree};

/// Fades `target` in (`Opacity` 0 -> 1, over the first stage's own duration) while
/// stepping its `Polygon` through `stages` in order, `stage_duration` ms each, all under
/// `seq` -- the core of every "starts as an invisible triangle, morphs into its real
/// shape" motif in this app (`ContentsItem`'s heptagon, `chrome.rs`'s brand mark). `target`
/// must already be spawned with a starting `Polygon` (`sides: 3.0, rounding: 0.0` by this
/// app's own convention) and `Opacity::new(0.0)` -- this only animates an existing entity,
/// it doesn't spawn one, so it composes with whatever the caller's own spawn already set
/// up (position, color, interaction).
///
/// Rotation is held at `rotation` across every stage -- this does *not* attempt
/// `navigator.rs`'s own much more elaborate per-stage overshoot/bounce/rotation
/// choreography (`build_morph`, spin-past-then-settle each stage); that's a specific
/// flourish on top of this core technique, not the technique itself, and pulling it in
/// here would make the common case (`morph_in(tree, e, seq, &[(7.0, 0.15)], 0.0, 0, 400)`)
/// carry weight it doesn't need. If a caller wants that flourish, it's still just more
/// `tree.animate(...)` calls `.during(seq)` alongside this one -- nothing here claims the
/// sequence exclusively.
///
/// How this gets *triggered* is deliberately not this function's concern: called directly
/// at spawn time (the common case) it just runs once; wrapped in
/// `tree.react::<C, _>(watch, move |_, _, mut tree: Tree| { morph_in(&mut tree, target,
/// tree.sequence(), &stages, rotation, 0, dur); })` it re-runs on every later write to
/// `C` -- the framework's own `react` is already the general "trigger an action on
/// change" door (see [Tree and Graft] in the book), so this doesn't invent a second one
/// beside it.
pub fn morph_in(
    tree: &mut Tree,
    target: Entity,
    seq: Entity,
    stages: &[(f32, f32)],
    rotation: f32,
    start: u64,
    stage_duration: u64,
) {
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .targeting(target)
            .during(seq)
            .start(start)
            .finish(start + stage_duration)
            .eased(Ease::Linear),
    );
    let mut t = start;
    for &(sides, rounding) in stages {
        let finish = t + stage_duration;
        tree.animate(
            Animation::new(Polygon {
                sides,
                rounding,
                rotation,
            })
            .targeting(target)
            .during(seq)
            .start(t)
            .finish(finish)
            .eased(Ease::DECELERATE),
        );
        t = finish;
    }
}
