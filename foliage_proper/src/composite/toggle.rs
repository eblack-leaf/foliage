use crate::Trigger;
use crate::{
    Animation, Color, Component, Ease, EcsExtension, Elevation, Entity, FocusBehavior, Grid,
    GridExt, InteractionListener, InteractionPropagation, LeafSprout, Location, OnClick, Panel,
    Rounding, Sequence, Sprout, Tree,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A toggle is one entity: components in ([`ToggleState`], [`ToggleStyle`]), [`Toggled`]
/// out. Clicks and programmatic `ToggleState` writes go through the same door -- Button's
/// `Engagement` pattern with persistent instead of momentary state.
#[derive(Component, Copy, Clone)]
pub struct Toggle {}
impl Toggle {
    pub fn new() -> ToggleSprout {
        ToggleSprout::default()
    }
}

/// Toggle's public value channel: write it to flip the switch programmatically.
#[derive(Component, Copy, Clone, Default)]
pub struct ToggleState(pub bool);

/// Toggle's OWN config vocabulary, poked as one unit:
/// `tree.write_to(toggle, ToggleStyle { .. })`.
#[derive(Component, Copy, Clone, Default)]
pub struct ToggleStyle {
    /// track fill while on
    pub on_fill: Color,
    /// track fill while off
    pub off_fill: Color,
    /// knob color, both states
    pub knob: Color,
}

/// Emitted at the toggle root whenever [`ToggleState`] changes (click or programmatic
/// write) -- including once at spawn with the initial value, since the reaction that fires
/// it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Toggled {
    pub on: bool,
}

/// The knob is a half-track-width pill inset 4px from the edges -- expressed relatively so
/// the widget reads as a switch at any size the author gives its `Location`.
fn knob_location(on: bool) -> Location {
    let horizontal = if on {
        50.pct()
            .as_left()
            .adjust(2)
            .with(100.pct().as_right().adjust(-4))
    } else {
        0.pct()
            .as_left()
            .adjust(4)
            .with(50.pct().as_right().adjust(-2))
    };
    Location::new().xs(
        horizontal,
        0.pct()
            .as_top()
            .adjust(4)
            .with(100.pct().as_bottom().adjust(-4)),
    )
}

#[derive(Default)]
pub struct ToggleSprout {
    leaf: LeafSprout,
    on: bool,
    style: ToggleStyle,
}
impl ToggleSprout {
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }
    pub fn colors(mut self, on_fill: Color, off_fill: Color, knob: Color) -> Self {
        self.style = ToggleStyle {
            on_fill,
            off_fill,
            knob,
        };
        self
    }
}
impl Sprout for ToggleSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Toggle {},
            ToggleState(self.on),
            self.style,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton
        let track = tree.branch(
            this,
            Panel::new()
                .rounding(Rounding::Full)
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );
        // no Location: state-dependent, set by that reaction's first fire
        let knob = tree.branch(
            this,
            Panel::new()
                .rounding(Rounding::Full)
                .elevate(Elevation::up(2))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );

        // input: click -> state flip. ONE component write; drawing happens in the reaction.
        tree.on_click(
            this,
            |trigger: Trigger<OnClick>, states: Query<&ToggleState>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.write_to(e, ToggleState(!states.get(e).unwrap().0));
            },
        );

        // render: state/style -> colors + knob placement. The first fire places the knob
        // directly (there's no prior Location to slide from); every later state change
        // slides it -- captured-flag state, the ProjectCard-image pattern.
        let mut place_directly = true;
        tree.react_any::<(ToggleState, ToggleStyle), _>(
            this,
            move |trigger: Trigger<Insert, (ToggleState, ToggleStyle)>,
                  states: Query<&ToggleState>,
                  styles: Query<&ToggleStyle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let on = states.get(e).unwrap().0;
                let style = *styles.get(e).unwrap();
                tree.write_to(track, if on { style.on_fill } else { style.off_fill });
                tree.write_to(knob, style.knob);
                if place_directly {
                    tree.write_to(knob, knob_location(on));
                    place_directly = false;
                } else {
                    Sequence::new(&mut tree).animate(
                        Animation::new(knob_location(on))
                            .targeting(knob)
                            .start(0)
                            .finish(150)
                            .eased(Ease::EMPHASIS),
                    );
                }
            },
        );
        // state -> event bridge; kept out of the restyle reaction so ToggleStyle writes
        // don't announce a state change that didn't happen.
        tree.react::<ToggleState, _>(
            this,
            |trigger: Trigger<Insert, ToggleState>, states: Query<&ToggleState>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.trigger_targets(Toggled::new(states.get(e).unwrap().0), e);
            },
        );
    }
}
