use crate::composite::Root;
use crate::Trigger;
use crate::{
    anchor, Anchor, Color, Component, CurrentInteraction, EcsExtension, Elevation, Entity,
    FocusBehavior, FontSize, Grid, GridExt, HorizontalAlignment, Icon, IconId, IconValue,
    InteractionListener, InteractionPropagation, LeafSprout, List, ListItems, Location, OnClick,
    Outline, Panel, Rounding, Sprout, Text, TextValue, Tree, Unfocused, VerticalAlignment,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Res;
use bevy_ecs::system::Query;

/// A dropdown is one entity: components in ([`DropdownOptions`], [`Selected`],
/// [`DropdownStyle`], [`DropdownConfig`], [`Expanded`]), [`SelectionChanged`] out.
///
/// Opinion (opt-in, like Button's icon-centering): options are **strings** -- the 90% case,
/// kept to one honest builder call. Authors needing rich option rows compose their own
/// trigger + [`List`]; both halves of this widget are public exactly so that's not a fork
/// of library internals.
///
/// The expanded option surface is a library [`List`] (long option sets scroll for free),
/// spawned as a TOP-LEVEL leaf anchored below the trigger -- branching it under the root
/// would clip it to the trigger's own rect (ancestor clipping is unconditional). That makes
/// it the sanctioned exception to slots-are-branches, torn down explicitly on collapse and
/// pinned at `Elevation::abs(90)` so it clears whatever context the dropdown sits in.
#[derive(Component, Copy, Clone)]
pub struct Dropdown {}
impl Dropdown {
    pub fn new() -> DropdownSprout {
        DropdownSprout::default()
    }
}

/// The option strings, rewritten as one unit to change the set.
#[derive(Component, Clone, Default)]
pub struct DropdownOptions(pub Vec<String>);
/// The selected option's index -- the dropdown's public value channel. Writes are clamped
/// onto the current option set.
#[derive(Component, Copy, Clone, Default)]
pub struct Selected(pub usize);
/// Open/closed state as a component (Button's `Engagement` pattern), so programmatic
/// open/close shares the trigger-click door.
#[derive(Component, Copy, Clone, Default)]
pub struct Expanded(pub bool);

/// Dropdown's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct DropdownStyle {
    /// option/selected text + chevron color
    pub foreground: Color,
    /// trigger + option-row fill
    pub background: Color,
    /// selected row highlight + expanded trigger accent
    pub accent: Color,
    /// trigger panel outline -- same knob `Panel` already exposes, just never forwarded here
    pub outline: Outline,
}
/// Structural config, separate from [`DropdownStyle`] so restyling doesn't re-state it.
#[derive(Component, Copy, Clone)]
pub struct DropdownConfig {
    /// expand indicator icon; the library ships none, so absent = no indicator
    pub chevron: Option<IconId>,
    /// option rows shown before the list scrolls
    pub max_visible: usize,
    /// expanded option row height in px -- forwarded straight to the `List` underneath
    pub row_height: i32,
    /// expanded option row gap in px -- forwarded straight to the `List` underneath
    pub row_gap: i32,
}
impl Default for DropdownConfig {
    fn default() -> Self {
        Self {
            chevron: None,
            max_visible: 5,
            row_height: 36,
            row_gap: 4,
        }
    }
}

/// Emitted at the dropdown root whenever [`Selected`] changes (row click or programmatic
/// write) -- including once at spawn with the initial value, since the reaction that fires
/// it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct SelectionChanged {
    pub index: usize,
}

/// Private child registry -- the collapse/select/unfocus paths all need these ids.
#[derive(Component, Copy, Clone)]
pub(crate) struct DropdownHandle {
    panel: Entity,
    text: Entity,
    chevron: Option<Entity>,
    options: Option<Entity>,
}

#[derive(Default)]
pub struct DropdownSprout {
    leaf: LeafSprout,
    options: Vec<String>,
    selected: usize,
    style: DropdownStyle,
    config: DropdownConfig,
}
impl DropdownSprout {
    pub fn options<S: Into<String>>(mut self, options: impl IntoIterator<Item = S>) -> Self {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
    pub fn chevron(mut self, icon: IconId) -> Self {
        self.config.chevron = Some(icon);
        self
    }
    pub fn max_visible(mut self, n: usize) -> Self {
        self.config.max_visible = n.max(1);
        self
    }
    pub fn row_height(mut self, px: i32) -> Self {
        self.config.row_height = px;
        self
    }
    pub fn row_gap(mut self, px: i32) -> Self {
        self.config.row_gap = px;
        self
    }
    pub fn colors(mut self, foreground: Color, background: Color, accent: Color) -> Self {
        self.style.foreground = foreground;
        self.style.background = background;
        self.style.accent = accent;
        self
    }
    pub fn outline(mut self, w: i32) -> Self {
        self.style.outline = Outline::new(w);
        self
    }
}
impl Sprout for DropdownSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Dropdown {},
            Selected(self.selected.min(self.options.len().saturating_sub(1))),
            DropdownOptions(self.options),
            Expanded(false),
            self.style,
            self.config,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton: trigger panel + selected-value text. Chevron and the option
        // surface are dynamic (config- and state-dependent respectively).
        let panel = tree.branch(
            this,
            Panel::new()
                .rounding(Rounding::Sm)
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
        // no Location: depends on whether a chevron is configured (right inset needs to
        // leave room for one only when it's actually there), set by the structure
        // reaction's first fire below.
        let text = tree.branch(
            this,
            Text::new("")
                .elevate(Elevation::up(2))
                .with((
                    VerticalAlignment::Middle,
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );
        tree.write_to(
            this,
            DropdownHandle {
                panel,
                text,
                chevron: None,
                options: None,
            },
        );
        // click the trigger -> flip expansion; the state write drives the rebuild below.
        tree.on_click(
            this,
            |trigger: Trigger<OnClick>, expanded: Query<&Expanded>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.write_to(e, Expanded(!expanded.get(e).unwrap().0));
            },
        );
        // structure/appearance, one door: expansion state, option set, colors, config.
        tree.react_any::<(Expanded, DropdownOptions, DropdownStyle, DropdownConfig), _>(
            this,
            move |trigger: Trigger<Insert, (Expanded, DropdownOptions, DropdownStyle, DropdownConfig)>,
                  expanded: Query<&Expanded>,
                  options: Query<&DropdownOptions>,
                  styles: Query<&DropdownStyle>,
                  configs: Query<&DropdownConfig>,
                  selected: Query<&Selected>,
                  mut handles: Query<&mut DropdownHandle>,
                  render_sizes: Res<crate::IconRenderSizes>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let open = expanded.get(e).unwrap().0;
                let opts = options.get(e).unwrap().clone();
                let style = *styles.get(e).unwrap();
                let config = *configs.get(e).unwrap();
                let current = selected
                    .get(e)
                    .unwrap()
                    .0
                    .min(opts.0.len().saturating_sub(1));
                let mut handle = handles.get_mut(e).unwrap();
                tree.write_to(handle.panel, (style.background, style.outline));
                // right inset reserves exactly the configured chevron's own registered
                // footprint (+ its own 8px right inset, + 4px breathing room) when one
                // exists -- never a guessed constant, since the chevron's size is itself
                // fully configurable per icon.
                let text_right_inset = match config.chevron {
                    Some(icon) => -(8 + render_sizes.get(icon).a() as i32 + 4),
                    None => -12,
                };
                tree.write_to(
                    handle.text,
                    (
                        TextValue(opts.0.get(current).cloned().unwrap_or_default()),
                        style.foreground,
                        Location::new().xs(
                            12.px().as_left().with(100.pct().as_right().adjust(text_right_inset)),
                            0.pct().as_top().with(100.pct().as_bottom()),
                        ),
                    ),
                );
                // chevron exists only while configured -- no defunct hidden entity. Its box
                // is exactly the icon's registered footprint (the clip region derives from
                // the box), right edge inset 8px, centered vertically.
                let chevron_location = |icon| {
                    let size = render_sizes.get(icon);
                    Location::new().xs(
                        100.pct()
                            .as_right()
                            .adjust(-8)
                            .with((size.a() as i32).px().as_width()),
                        50.pct()
                            .as_center_y()
                            .with((size.b() as i32).px().as_height()),
                    )
                };
                match (config.chevron, handle.chevron) {
                    (Some(icon), Some(existing)) => {
                        tree.write_to(
                            existing,
                            (IconValue(icon), style.foreground, chevron_location(icon)),
                        );
                    }
                    (Some(icon), None) => {
                        let chevron = tree.branch(
                            e,
                            Icon::new(icon)
                                .color(style.foreground)
                                .at(chevron_location(icon))
                                .elevate(Elevation::up(2))
                                .with((
                                    InteractionPropagation::pass_through(),
                                    FocusBehavior::ignore(),
                                )),
                        );
                        handle.chevron = Some(chevron);
                    }
                    (None, Some(existing)) => {
                        tree.remove(existing);
                        handle.chevron = None;
                    }
                    (None, None) => {}
                }
                // the option surface: rebuilt fresh on every fire while open, gone while
                // closed. Rows write (Selected, Expanded(false)) at the root -- selection
                // and collapse through the same doors as everything else.
                if let Some(existing) = handle.options.take() {
                    tree.remove(existing);
                }
                if open && !opts.0.is_empty() {
                    let shown = opts.0.len().min(config.max_visible);
                    let height = shown as i32 * (config.row_height + config.row_gap) + config.row_gap;
                    let row_options = opts.0.clone();
                    let surface = tree.leaf(
                        List::new()
                            .items(ListItems::new(row_options.len(), move |tree, slot, i| {
                                let row = tree.branch(
                                    slot,
                                    Panel::new()
                                        .rounding(Rounding::Sm)
                                        .color(if i == current {
                                            style.accent
                                        } else {
                                            style.background
                                        })
                                        .at(Location::new().xs(
                                            0.pct().as_left().with(100.pct().as_right()),
                                            0.pct().as_top().with(100.pct().as_bottom()),
                                        ))
                                        .elevate(Elevation::up(1))
                                        .with((InteractionListener::new(), Root(e))),
                                );
                                tree.branch(
                                    slot,
                                    Text::new(row_options[i].clone())
                                        .color(style.foreground)
                                        .at(Location::new().xs(
                                            12.px().as_left().with(100.pct().as_right()),
                                            0.pct().as_top().with(100.pct().as_bottom()),
                                        ))
                                        .elevate(Elevation::up(2))
                                        .with((
                                            VerticalAlignment::Middle,
                                            InteractionPropagation::pass_through(),
                                            FocusBehavior::ignore(),
                                        )),
                                );
                                tree.on_click(row, move |_: Trigger<OnClick>, mut tree: Tree| {
                                    tree.write_to(e, (Selected(i), Expanded(false)));
                                });
                            }))
                            .row_height(config.row_height)
                            .gap(config.row_gap)
                            .at(Location::new().xs(
                                anchor().left().as_left().with(anchor().right().as_right()),
                                anchor()
                                    .bottom()
                                    .as_top()
                                    .adjust(4)
                                    .with(height.px().as_height()),
                            ))
                            .elevate(Elevation::abs(90))
                            .with((Anchor::new(e), style.background, Panel::default(), Root(e))),
                    );
                    handle.options = Some(surface);
                }
            },
        );
        // selection -> trigger text + event bridge; kept out of the rebuild reaction so
        // style/config writes don't announce a selection that didn't change. The rebuild
        // reaction reads Selected too (row highlight), but only Selected writes land here.
        tree.react::<Selected, _>(
            this,
            |trigger: Trigger<Insert, Selected>,
             selected: Query<&Selected>,
             options: Query<&DropdownOptions>,
             handles: Query<&DropdownHandle>,
             mut tree: Tree| {
                let e = trigger.event_target();
                let opts = options.get(e).unwrap();
                let index = selected
                    .get(e)
                    .unwrap()
                    .0
                    .min(opts.0.len().saturating_sub(1));
                if let Some(value) = opts.0.get(index) {
                    tree.write_to(handles.get(e).unwrap().text, TextValue(value.clone()));
                }
                tree.trigger_targets(SelectionChanged::new(index), e);
            },
        );
        // click-away: collapse when focus leaves the widget -- unless it landed on our own
        // option surface (the rows carry Root(this); the List root is tracked directly).
        tree.subscribe(
            this,
            move |trigger: Trigger<Unfocused>,
                  current_interaction: Res<CurrentInteraction>,
                  roots: Query<&Root>,
                  handles: Query<&DropdownHandle>,
                  expanded: Query<&Expanded>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                if !expanded.get(e).unwrap().0 {
                    return;
                }
                if let Some(f) = current_interaction.focused {
                    let handle = handles.get(e).unwrap();
                    if f == e || Some(f) == handle.options || Root::resolve(f, &roots) == e {
                        return;
                    }
                }
                tree.write_to(e, Expanded(false));
            },
        );
    }
}
