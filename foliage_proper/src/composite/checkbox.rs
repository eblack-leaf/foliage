use crate::Trigger;
use crate::{
    Color, Component, EcsExtension, Elevation, Entity, FocusBehavior, Grid, GridExt,
    InteractionListener, InteractionPropagation, LeafSprout, Line, Location, OnClick, Outline,
    Panel, Rounding, Sprout, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A checkbox is one entity: components in ([`CheckboxState`], [`CheckboxStyle`]),
/// [`Checked`] out. Same `Engagement`-free persistent-state shape as `Toggle` (click flips
/// one component, a reaction draws the result) -- the visual difference is the whole point:
/// an empty outlined box at rest, a filled box + checkmark once checked.
#[derive(Component, Copy, Clone)]
pub struct Checkbox {}
impl Checkbox {
    pub fn new() -> CheckboxSprout {
        CheckboxSprout::default()
    }
}

/// Checkbox's public value channel: write it to check/uncheck programmatically.
#[derive(Component, Copy, Clone, Default)]
pub struct CheckboxState(pub bool);

/// Checkbox's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct CheckboxStyle {
    /// box border color while unchecked
    pub outline: Color,
    /// box fill color while checked
    pub fill: Color,
    /// checkmark stroke color
    pub check: Color,
    pub rounding: Rounding,
}

/// Emitted at the checkbox root whenever [`CheckboxState`] changes (click or programmatic
/// write) -- including once at spawn with the initial value, since the reaction that fires
/// it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Checked {
    pub on: bool,
}

#[derive(Default)]
pub struct CheckboxSprout {
    leaf: LeafSprout,
    on: bool,
    style: CheckboxStyle,
}
impl CheckboxSprout {
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }
    pub fn colors(mut self, outline: Color, fill: Color, check: Color) -> Self {
        self.style.outline = outline;
        self.style.fill = fill;
        self.style.check = check;
        self
    }
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.style.rounding = r;
        self
    }
}
impl Sprout for CheckboxSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Checkbox {},
            CheckboxState(self.on),
            self.style,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton
        let panel = tree.branch(
            this,
            Panel::new()
                .outline(2)
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
        // the checkmark: two short strokes, drawn from scratch (no icon asset dependency) --
        // hidden while unchecked, shown over the filled box once checked.
        let stroke_a = tree.branch(
            this,
            Line::new(2)
                .at(Location::new().xs(
                    22.pct().as_x().with(52.pct().as_y()),
                    40.pct().as_x().with(74.pct().as_y()),
                ))
                .elevate(Elevation::up(2))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );
        let stroke_b = tree.branch(
            this,
            Line::new(2)
                .at(Location::new().xs(
                    40.pct().as_x().with(74.pct().as_y()),
                    80.pct().as_x().with(24.pct().as_y()),
                ))
                .elevate(Elevation::up(2))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );

        tree.on_click(
            this,
            |trigger: Trigger<OnClick>, states: Query<&CheckboxState>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.write_to(e, CheckboxState(!states.get(e).unwrap().0));
            },
        );

        // render: state/style -> box fill-vs-outline + checkmark visibility. Panel's own
        // fill/outline exclusivity does the state switch for free: outline-only (no fill)
        // while unchecked, solid fill (no outline) while checked -- see `panel/mod.rs`, a
        // positive `Outline` value means stroke-only, `Outline::default()` (-1) means solid.
        tree.react_any::<(CheckboxState, CheckboxStyle), _>(
            this,
            move |trigger: Trigger<Insert, (CheckboxState, CheckboxStyle)>,
                  states: Query<&CheckboxState>,
                  styles: Query<&CheckboxStyle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let on = states.get(e).unwrap().0;
                let style = *styles.get(e).unwrap();
                if on {
                    tree.write_to(panel, (style.fill, Outline::default(), style.rounding));
                } else {
                    tree.write_to(panel, (style.outline, Outline::new(2), style.rounding));
                }
                tree.write_to(stroke_a, (style.check, Visibility::new(on)));
                tree.write_to(stroke_b, (style.check, Visibility::new(on)));
            },
        );
        // state -> event bridge; kept out of the render reaction so style-only writes don't
        // announce a state change that didn't happen.
        tree.react::<CheckboxState, _>(
            this,
            |trigger: Trigger<Insert, CheckboxState>,
             states: Query<&CheckboxState>,
             mut tree: Tree| {
                let e = trigger.event_target();
                tree.trigger_targets(Checked::new(states.get(e).unwrap().0), e);
            },
        );
    }
}
