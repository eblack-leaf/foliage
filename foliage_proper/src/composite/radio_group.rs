use crate::Trigger;
use crate::{
    Color, Component, EcsExtension, Elevation, Entity, Grid, GridExt, HorizontalAlignment,
    InteractionListener, LeafSprout, Location, OnClick, Outline, Panel, Sprout, Text, Tree,
    VerticalAlignment,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A radio group is one entity: components in ([`RadioOptions`], [`RadioSelected`],
/// [`RadioStyle`]), [`RadioChanged`] out. Exclusivity ("picking one deselects the rest") is
/// owned centrally the same way `Dropdown` owns `Selected` across its own option set --
/// there is deliberately no standalone single-radio widget, since a lone radio button with
/// no group to be exclusive within isn't a meaningful thing to build.
#[derive(Component, Copy, Clone)]
pub struct RadioGroup {}
impl RadioGroup {
    pub fn new() -> RadioGroupSprout {
        RadioGroupSprout::default()
    }
}

/// The option labels, rewritten as one unit to change the set.
#[derive(Component, Clone, Default)]
pub struct RadioOptions(pub Vec<String>);
/// The selected option's index -- the group's public value channel. Writes are clamped onto
/// the current option set.
#[derive(Component, Copy, Clone, Default)]
pub struct RadioSelected(pub usize);

/// Radio group's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct RadioStyle {
    /// selected circle's fill + label color
    pub active: Color,
    /// every other circle's outline + label color
    pub inactive: Color,
}

/// Emitted at the group's root whenever [`RadioSelected`] changes (row click or
/// programmatic write) -- including once at spawn with the initial value, since the
/// reaction that fires it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct RadioChanged {
    pub index: usize,
}

const CIRCLE_SIZE: i32 = 20;

/// Private child registry: the patch reaction and click handlers need the stable entities
/// the structure reaction built.
#[derive(Component, Clone, Default)]
pub(crate) struct RadioHandle {
    rows: Vec<(Entity, Entity)>, // (circle, label) per option
}

pub struct RadioGroupSprout {
    leaf: LeafSprout,
    options: Vec<String>,
    selected: usize,
    style: RadioStyle,
}
impl Default for RadioGroupSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            options: Vec::new(),
            selected: 0,
            style: RadioStyle::default(),
        }
    }
}
impl RadioGroupSprout {
    pub fn options<S: Into<String>>(mut self, options: impl IntoIterator<Item = S>) -> Self {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
    pub fn colors(mut self, active: Color, inactive: Color) -> Self {
        self.style.active = active;
        self.style.inactive = inactive;
        self
    }
}
impl Sprout for RadioGroupSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            RadioGroup {},
            RadioSelected(self.selected.min(self.options.len().saturating_sub(1))),
            RadioOptions(self.options),
            self.style,
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        tree.write_to(this, RadioHandle::default());

        // STRUCTURE: rebuild row entities only when the option set or style changes.
        tree.react_any::<(RadioOptions, RadioStyle), _>(
            this,
            move |trigger: Trigger<Insert, (RadioOptions, RadioStyle)>,
                  options: Query<&RadioOptions>,
                  styles: Query<&RadioStyle>,
                  selected: Query<&RadioSelected>,
                  mut handles: Query<&mut RadioHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let opts = options.get(e).unwrap().clone();
                let style = *styles.get(e).unwrap();
                let current = selected
                    .get(e)
                    .unwrap()
                    .0
                    .min(opts.0.len().saturating_sub(1));
                let mut handle = handles.get_mut(e).unwrap();
                for (circle, label) in handle.rows.drain(..) {
                    tree.remove(circle);
                    tree.remove(label);
                }
                if opts.0.is_empty() {
                    return;
                }
                tree.write_to(e, Grid::new(1.col(), opts.0.len().row().gap(4)));
                for (i, text) in opts.0.iter().enumerate() {
                    let row = (i + 1) as i32;
                    let active = i == current;
                    let circle = tree.branch(
                        e,
                        Panel::new()
                            .rounding(crate::Rounding::Full)
                            .color(if active { style.active } else { style.inactive })
                            .outline(if active { -1 } else { 2 })
                            .at(Location::new().xs(
                                0.px().as_left().with(CIRCLE_SIZE.px().as_width()),
                                row.row().as_center_y().with(CIRCLE_SIZE.px().as_height()),
                            ))
                            .elevate(Elevation::up(1))
                            .with(InteractionListener::new()),
                    );
                    // clickable too, not just the circle -- the label is the bigger, more
                    // natural tap target for this kind of row.
                    let label = tree.branch(
                        e,
                        Text::new(text.clone())
                            .color(if active { style.active } else { style.inactive })
                            .at(Location::new().xs(
                                (CIRCLE_SIZE + 8).px().as_left().with(100.pct().as_right()),
                                row.row().as_top().with(row.row().as_bottom()),
                            ))
                            .elevate(Elevation::up(1))
                            .with((
                                VerticalAlignment::Middle,
                                HorizontalAlignment::Left,
                                InteractionListener::new(),
                            )),
                    );
                    tree.on_click(circle, move |_: Trigger<OnClick>, mut tree: Tree| {
                        tree.write_to(e, RadioSelected(i));
                    });
                    tree.on_click(label, move |_: Trigger<OnClick>, mut tree: Tree| {
                        tree.write_to(e, RadioSelected(i));
                    });
                    handle.rows.push((circle, label));
                }
            },
        );
        // PATCH: selection changes recolor the stable rows and announce -- no spawns, no
        // removes.
        tree.react::<RadioSelected, _>(
            this,
            move |trigger: Trigger<Insert, RadioSelected>,
                  selected: Query<&RadioSelected>,
                  options: Query<&RadioOptions>,
                  styles: Query<&RadioStyle>,
                  handles: Query<&RadioHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let opts = options.get(e).unwrap();
                let current = selected
                    .get(e)
                    .unwrap()
                    .0
                    .min(opts.0.len().saturating_sub(1));
                let style = *styles.get(e).unwrap();
                let handle = handles.get(e).unwrap();
                for (i, (circle, label)) in handle.rows.iter().enumerate() {
                    let active = i == current;
                    let color = if active { style.active } else { style.inactive };
                    tree.write_to(*circle, (color, Outline::new(if active { -1 } else { 2 })));
                    tree.write_to(*label, color);
                }
                tree.trigger_targets(RadioChanged::new(current), e);
            },
        );
    }
}
