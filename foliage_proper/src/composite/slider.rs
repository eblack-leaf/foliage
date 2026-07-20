use crate::Trigger;
use crate::composite::Progress;
use crate::{
    Anchor, Color, Component, CurrentInteraction, Dragged, EcsExtension, Elevation, Entity,
    FocusBehavior, Grid, GridExt, InteractionListener, InteractionPropagation, LeafSprout, Line,
    Location, Logical, OnClick, Panel, Rounding, Section, Sprout, Tree, Visibility, anchor,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Res;
use bevy_ecs::system::Query;

/// A slider is one entity: components in ([`Progress`], [`SliderStyle`], [`SliderBehavior`]),
/// [`ProgressChanged`] out. Drag, tap-to-seek, and programmatic `Progress` writes all go
/// through the same `Progress` door, so consumers can't tell (and don't care) which one
/// moved it. `.interactive(false)` is the display-only progress bar: knob hidden, all
/// interaction disabled, `Progress` writes still render.
#[derive(Component, Copy, Clone)]
pub struct Slider {}
impl Slider {
    pub fn new() -> SliderSprout {
        SliderSprout::default()
    }
}

/// Slider's OWN config vocabulary, poked as one unit:
/// `tree.write_to(slider, SliderStyle { .. })`.
#[derive(Component, Copy, Clone)]
pub struct SliderStyle {
    /// the full-length line behind the fill
    pub track: Color,
    /// elapsed-portion line + knob color
    pub fill: Color,
    /// track/fill line thickness in px
    pub weight: i32,
    /// knob diameter in px
    pub knob_size: i32,
}
impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track: Color::default(),
            fill: Color::default(),
            weight: 4,
            knob_size: 16,
        }
    }
}

/// Separate from [`SliderStyle`] because it isn't appearance: flipping this at runtime
/// (e.g. a media player locking the timeline while buffering) shouldn't require re-stating
/// colors.
#[derive(Component, Copy, Clone)]
pub struct SliderBehavior {
    pub interactive: bool,
}
impl Default for SliderBehavior {
    fn default() -> Self {
        Self { interactive: true }
    }
}

/// Emitted at the slider root whenever [`Progress`] changes (drag, tap, or programmatic
/// write) -- including once at spawn with the initial value, since the reaction that fires
/// it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct ProgressChanged {
    pub progress: f32,
}

#[derive(Default)]
pub struct SliderSprout {
    leaf: LeafSprout,
    progress: f32,
    style: SliderStyle,
    behavior: SliderBehavior,
}
impl SliderSprout {
    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }
    pub fn colors(mut self, track: Color, fill: Color) -> Self {
        self.style.track = track;
        self.style.fill = fill;
        self
    }
    pub fn weight(mut self, w: i32) -> Self {
        self.style.weight = w;
        self
    }
    pub fn knob_size(mut self, s: i32) -> Self {
        self.style.knob_size = s;
        self
    }
    pub fn interactive(mut self, i: bool) -> Self {
        self.behavior.interactive = i;
        self
    }
}
impl Sprout for SliderSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Slider {},
            Progress(self.progress),
            self.style,
            self.behavior,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton -- weights/colors/knob size are style-dependent and land via the
        // style reaction's first fire; the placeholder weight never renders.
        let track = tree.branch(
            this,
            Line::new(1)
                .at(Location::new().xs(
                    0.pct().as_x().with(50.pct().as_y()),
                    100.pct().as_x().with(50.pct().as_y()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );
        // no Location: progress-dependent, set by that reaction's first fire
        let fill = tree.branch(
            this,
            Line::new(1).elevate(Elevation::up(2)).with((
                InteractionPropagation::pass_through(),
                FocusBehavior::ignore(),
            )),
        );
        let knob = tree.branch(
            this,
            Panel::new()
                .rounding(Rounding::Full)
                .elevate(Elevation::up(3))
                .with((
                    Anchor::new(fill),
                    InteractionListener::new(),
                    InteractionPropagation::grab().disable_drag(),
                    FocusBehavior::ignore(),
                )),
        );

        // input: drag -> value. Touches ONE component; drawing happens in the reaction, so
        // drag and programmatic writes render identically.
        tree.subscribe(
            knob,
            move |_: Trigger<Dragged>,
                  interaction: Res<CurrentInteraction>,
                  sections: Query<&Section<Logical>>,
                  mut tree: Tree| {
                let bounds = sections.get(this).unwrap();
                let pct = ((interaction.click().current.left() - bounds.left()) / bounds.width())
                    .clamp(0.0, 1.0);
                tree.write_to(this, Progress(pct));
            },
        );
        // input: tap-to-seek anywhere on the track. The root listener is disabled entirely
        // when non-interactive (see the behavior arm below), so no gate is needed here.
        tree.on_click(
            this,
            move |trigger: Trigger<OnClick>,
                  interaction: Res<CurrentInteraction>,
                  sections: Query<&Section<Logical>>,
                  mut tree: Tree| {
                let bounds = sections.get(trigger.event_target()).unwrap();
                let pct = ((interaction.click().current.left() - bounds.left()) / bounds.width())
                    .clamp(0.0, 1.0);
                tree.write_to(trigger.event_target(), Progress(pct));
            },
        );

        // render: value -> geometry. The knob is centered on fill's endpoint with no inset
        // of its own (see the style reaction below), so fill's endpoint itself carries a
        // px correction that shrinks linearly from +half-knob at value=0 to -half-knob at
        // value=1 (and vanishes at the midpoint) -- the closed-form equivalent of mapping
        // value onto an (inset, track_width - inset) range instead of the full (0, 100%)
        // one, expressed as a pct-plus-px adjustment since the two can't be combined any
        // other way here. Without it the knob clips past the track's own edges at the
        // extremes (half the knob renders outside the parent rect and gets cut off).
        tree.react::<Progress, _>(
            this,
            move |trigger: Trigger<Insert, Progress>,
                  progress: Query<&Progress>,
                  styles: Query<&SliderStyle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let value = progress.get(e).unwrap().0.clamp(0.0, 1.0);
                let inset = styles.get(e).unwrap().knob_size as f32 / 2.0;
                let correction = (inset * (1.0 - 2.0 * value)) as i32;
                tree.write_to(
                    fill,
                    Location::new().xs(
                        0.pct().as_x().with(50.pct().as_y()),
                        (value * 100.0)
                            .pct()
                            .as_x()
                            .adjust(correction)
                            .with(50.pct().as_y()),
                    ),
                );
                tree.trigger_targets(ProgressChanged::new(value), e);
            },
        );
        // appearance + interactivity, one door for config writes and initial state
        tree.react_any::<(SliderStyle, SliderBehavior), _>(
            this,
            move |trigger: Trigger<Insert, (SliderStyle, SliderBehavior)>,
                  styles: Query<&SliderStyle>,
                  behaviors: Query<&SliderBehavior>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let style = *styles.get(e).unwrap();
                tree.write_to(track, (Line::new_marker(style.weight), style.track));
                tree.write_to(fill, (Line::new_marker(style.weight), style.fill));
                tree.write_to(
                    knob,
                    (
                        style.fill,
                        Location::new().xs(
                            anchor()
                                .right()
                                .as_center_x()
                                .with(style.knob_size.px().as_width()),
                            50.pct()
                                .as_center_y()
                                .with(style.knob_size.px().as_height()),
                        ),
                    ),
                );
                // disable at the root: the cascade reaches the knob's listener too, and
                // re-enabling restores both -- display-only sliders swallow no clicks.
                if behaviors.get(e).unwrap().interactive {
                    tree.enable(e);
                    tree.write_to(knob, Visibility::new(true));
                } else {
                    tree.disable(e);
                    tree.write_to(knob, Visibility::new(false));
                }
            },
        );
    }
}
