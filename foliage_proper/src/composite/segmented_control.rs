use crate::composite::Root;
use crate::Trigger;
use crate::{
    Color, Component, EcsExtension, Elevation, Entity, FocusBehavior, Grid, GridExt,
    HorizontalAlignment, InteractionListener, InteractionPropagation, LeafSprout, Location,
    OnClick, Panel, Rounding, Sprout, Text, Tree, VerticalAlignment,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A segmented control is one entity: components in ([`SegmentedOptions`],
/// [`SegmentedSelected`], [`SegmentedStyle`]), [`SegmentChanged`] out. A row of exclusive
/// options rendered inline (not collapsed behind a trigger the way `Dropdown` is) -- the
/// same shape as `Pagination`'s `Numbered` mode, just labeled by the author instead of page
/// numbers, and with no paging concept at all.
#[derive(Component, Copy, Clone)]
pub struct SegmentedControl {}
impl SegmentedControl {
    pub fn new() -> SegmentedControlSprout {
        SegmentedControlSprout::default()
    }
}

/// The option strings, rewritten as one unit to change the set.
#[derive(Component, Clone, Default)]
pub struct SegmentedOptions(pub Vec<String>);
/// The selected option's index -- the control's public value channel. Writes are clamped
/// onto the current option set.
#[derive(Component, Copy, Clone, Default)]
pub struct SegmentedSelected(pub usize);

/// Segmented control's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct SegmentedStyle {
    /// selected segment's fill + label color
    pub active: Color,
    /// every other segment's label color
    pub inactive: Color,
    pub rounding: Rounding,
}

/// Emitted at the control's root whenever [`SegmentedSelected`] changes (segment click or
/// programmatic write) -- including once at spawn with the initial value, since the
/// reaction that fires it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct SegmentChanged {
    pub index: usize,
}

/// Private child registry: the patch reaction and click handlers need the stable segment
/// entities the structure reaction built.
#[derive(Component, Clone, Default)]
pub(crate) struct SegmentedHandle {
    segments: Vec<(Entity, Entity)>, // (panel, label) per option
}

pub struct SegmentedControlSprout {
    leaf: LeafSprout,
    options: Vec<String>,
    selected: usize,
    style: SegmentedStyle,
}
impl Default for SegmentedControlSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            options: Vec::new(),
            selected: 0,
            style: SegmentedStyle::default(),
        }
    }
}
impl SegmentedControlSprout {
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
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.style.rounding = r;
        self
    }
}
impl Sprout for SegmentedControlSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            SegmentedControl {},
            SegmentedSelected(self.selected.min(self.options.len().saturating_sub(1))),
            SegmentedOptions(self.options),
            self.style,
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        tree.write_to(this, SegmentedHandle::default());

        // STRUCTURE: rebuild segment entities only when the option set or style changes --
        // mirrors Pagination's structure/patch split exactly.
        tree.react_any::<(SegmentedOptions, SegmentedStyle), _>(
            this,
            move |trigger: Trigger<Insert, (SegmentedOptions, SegmentedStyle)>,
                  options: Query<&SegmentedOptions>,
                  styles: Query<&SegmentedStyle>,
                  selected: Query<&SegmentedSelected>,
                  mut handles: Query<&mut SegmentedHandle>,
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
                for (panel, label) in handle.segments.drain(..) {
                    tree.remove(panel);
                    tree.remove(label);
                }
                if opts.0.is_empty() {
                    return;
                }
                tree.write_to(
                    e,
                    Grid::new(opts.0.len().col().gap(4), 1.row()),
                );
                for (i, text) in opts.0.iter().enumerate() {
                    let panel = tree.branch(
                        e,
                        Panel::new()
                            .rounding(style.rounding)
                            .color(if i == current {
                                style.active
                            } else {
                                style.inactive
                            })
                            .at(Location::new().xs(
                                (i + 1).col().as_left().with((i + 1).col().as_right()),
                                1.row().as_top().with(1.row().as_bottom()),
                            ))
                            .elevate(Elevation::up(1))
                            .with((InteractionListener::new(), Root(e))),
                    );
                    let label = tree.branch(
                        e,
                        Text::new(text.clone())
                            .color(if i == current {
                                style.active
                            } else {
                                style.inactive
                            })
                            .at(Location::new().xs(
                                (i + 1).col().as_left().with((i + 1).col().as_right()),
                                1.row().as_top().with(1.row().as_bottom()),
                            ))
                            .elevate(Elevation::up(2))
                            .with((
                                HorizontalAlignment::Center,
                                VerticalAlignment::Middle,
                                InteractionPropagation::pass_through(),
                                FocusBehavior::ignore(),
                            )),
                    );
                    tree.on_click(panel, move |_: Trigger<OnClick>, mut tree: Tree| {
                        tree.write_to(e, SegmentedSelected(i));
                    });
                    handle.segments.push((panel, label));
                }
            },
        );
        // PATCH: selection changes recolor the stable segments and announce -- no spawns, no
        // removes.
        tree.react::<SegmentedSelected, _>(
            this,
            move |trigger: Trigger<Insert, SegmentedSelected>,
                  selected: Query<&SegmentedSelected>,
                  options: Query<&SegmentedOptions>,
                  styles: Query<&SegmentedStyle>,
                  handles: Query<&SegmentedHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let opts = options.get(e).unwrap();
                let current = selected.get(e).unwrap().0.min(opts.0.len().saturating_sub(1));
                let style = *styles.get(e).unwrap();
                let handle = handles.get(e).unwrap();
                for (i, (panel, label)) in handle.segments.iter().enumerate() {
                    let active = i == current;
                    tree.write_to(*panel, if active { style.active } else { style.inactive });
                    tree.write_to(*label, if active { style.active } else { style.inactive });
                }
                tree.trigger_targets(SegmentChanged::new(current), e);
            },
        );
    }
}
