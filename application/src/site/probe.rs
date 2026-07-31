//! Throwaway on-screen scroll instrumentation.
//!
//! Delete this module, its `mod` line, the `Scroller` marker on `shell::content_area`, and the
//! `attach` call once the scroll question is settled. It exists because the numbers have to be
//! read on a device where a log is not reachable.
//!
//! Pinned to the viewport as a root-level leaf, the way `drawer`'s scrim is -- inside the
//! router it would scroll away with the content it is measuring.

use foliage::{
    Color, CurrentInteraction, EcsExtension, Elevation, Entity, FontSize, GridExt,
    HorizontalAlignment, Location, Panel, Query, Res, Rounding, Sprout, Text, Tree,
    VerticalAlignment, View, component,
};

use crate::site::{role, space, type_scale};

/// Marks the scroll container, so the probe can find the one `View` that actually moves.
#[component]
#[derive(Copy, Clone)]
pub(crate) struct Scroller;

const H: i32 = 18;

#[component]
pub(crate) struct Probe {
    /// Last offset seen, to difference against.
    last: Option<f32>,
    /// px the view moved on the previous tick.
    per_tick: f32,
    /// Largest px/tick seen since the last press -- a coast's peak is over in a few frames
    /// and a live number is unreadable at that speed.
    peak: f32,
    /// Velocity handed off at the last release, px/ms. No latch needed --
    /// `interactive_elements` resets `CurrentInteraction::velocity` only on `Start`, so after
    /// a release it holds the exact value `Coasting` was given until the next touch.
    release: f32,
    shown: Option<String>,
}

pub(crate) fn attach(foliage: &mut foliage::Foliage) {
    foliage.user.add_systems(drive);
}

/// A line pinned to the bottom of the viewport.
pub(crate) fn build(tree: &mut Tree) {
    let backing = tree.leaf(
        Panel::new()
            .color(role::surface())
            .rounding(Rounding::None)
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                100.pct().as_bottom().with(H.px().as_height()),
            ))
            .elevate(Elevation::up(9)),
    );
    tree.branch(
        backing,
        Text::new("scroll  --")
            .size(FontSize::new(type_scale::LABEL))
            .color(Color::amber(400))
            .at(Location::new().xs(
                space::SM.px().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::up(10))
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                Probe {
                    last: None,
                    per_tick: 0.0,
                    peak: 0.0,
                    release: 0.0,
                    shown: None,
                },
            )),
    );
}

fn drive(
    current: Res<CurrentInteraction>,
    scrollers: Query<&View, foliage::With<Scroller>>,
    mut probes: Query<(Entity, &mut Probe)>,
    mut tree: Tree,
) {
    let Ok(view) = scrollers.single() else {
        return;
    };
    let offset = view.offset().top();
    for (entity, mut probe) in probes.iter_mut() {
        probe.release = current.velocity().top();
        // `velocity` is zeroed on `Start`, so this is "a new touch began" -- without it `peak`
        // holds the first flick's number forever and every later one reads as smaller.
        if probe.release == 0.0 {
            probe.peak = 0.0;
        }

        if let Some(last) = probe.last {
            probe.per_tick = offset - last;
            if probe.per_tick.abs() > probe.peak.abs() {
                probe.peak = probe.per_tick;
            }
        }
        probe.last = Some(offset);

        let line = format!(
            "v {:>6.2}/ms   now {:>6.1}   peak {:>6.1}   off {:>6.0}",
            probe.release, probe.per_tick, probe.peak, offset
        );
        if probe.shown.as_deref() == Some(line.as_str()) {
            continue;
        }
        probe.shown = Some(line.clone());
        tree.write_to(entity, Text { value: line });
    }
}
