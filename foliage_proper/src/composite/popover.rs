use crate::composite::{Root, SlotFn};
use crate::Trigger;
use crate::{
    anchor, Anchor, Color, Component, CurrentInteraction, EcsExtension, Elevation, Entity, Grid,
    GridExt, InteractionListener, InteractionPropagation, Leaf, LeafSprout, Location,
    LocationValue, OnClick, Panel, Sprout, Tree, Unfocused,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Res;
use bevy_ecs::system::Query;
use std::sync::Arc;

/// A popover is one entity: components in ([`PopoverConfig`], [`PopoverExpanded`],
/// [`PopoverStyle`]), [`PopoverOpened`]/[`PopoverClosed`] out. Tap-triggered, not
/// hover-triggered -- this interaction model has no hover concept at all (touch has none),
/// so this is the same click/expand/click-away shape `Dropdown` already uses, generalized
/// from a string list to arbitrary author content via the slot convention (see
/// [`crate::composite::SlotFn`]) for both halves: what's always visible (the trigger) and
/// what shows while expanded (the content).
///
/// Constraint worth knowing: the trigger slot itself passes interaction through to this
/// entity's own click handler (the same way `Dropdown`'s trigger panel/text do), but
/// whatever the author's own `.trigger(..)` closure builds does not automatically inherit
/// that -- if the author's trigger content is itself `InteractionListener`-bearing (a
/// button, say) without its own `InteractionPropagation::pass_through()`, it will consume
/// the click and the popover won't open. Plain content (`Text`/`Icon`/`Panel` with no
/// listener of their own) works with no extra care.
#[derive(Component, Copy, Clone)]
pub struct Popover {}
impl Popover {
    pub fn new() -> PopoverSprout {
        PopoverSprout {
            leaf: LeafSprout::default(),
            trigger: None,
            content: None,
            style: PopoverStyle::default(),
            placement: PopoverPlacement::Below,
            extent: None,
        }
    }
}

/// Which side of the trigger the content opens on -- the actual value this composite adds
/// over hand-rolled `anchor()` math: it knows the trigger's real position, the author just
/// picks a direction. The axis *across* the trigger (width for Below/Above, height for
/// Left/Right) always matches the trigger's own real size -- the same real relationship
/// `Dropdown`'s option list already uses for its own trigger-width match, not a guess. Only
/// the axis *away* from the trigger (how far the content reaches) is the author's call,
/// via `.extent(..)` -- genuinely irreducible, since nothing knows arbitrary content's own
/// size but the author who built it.
#[derive(Copy, Clone, Default)]
pub enum PopoverPlacement {
    #[default]
    Below,
    Above,
    Left,
    Right,
}
impl PopoverPlacement {
    const GAP: i32 = 4;
    fn location(self, extent: LocationValue) -> Location {
        match self {
            PopoverPlacement::Below => Location::new().xs(
                anchor().left().as_left().with(anchor().right().as_right()),
                anchor()
                    .bottom()
                    .as_top()
                    .adjust(Self::GAP)
                    .with(extent.as_height()),
            ),
            PopoverPlacement::Above => Location::new().xs(
                anchor().left().as_left().with(anchor().right().as_right()),
                anchor()
                    .top()
                    .as_bottom()
                    .adjust(-Self::GAP)
                    .with(extent.as_height()),
            ),
            PopoverPlacement::Left => Location::new().xs(
                anchor()
                    .left()
                    .as_right()
                    .adjust(-Self::GAP)
                    .with(extent.as_width()),
                anchor().top().as_top().with(anchor().bottom().as_bottom()),
            ),
            PopoverPlacement::Right => Location::new().xs(
                anchor()
                    .right()
                    .as_left()
                    .adjust(Self::GAP)
                    .with(extent.as_width()),
                anchor().top().as_top().with(anchor().bottom().as_bottom()),
            ),
        }
    }
}

/// Open/closed state as a component (`Toggle`'s persistent-state pattern), so programmatic
/// open/close shares the trigger-click door.
#[derive(Component, Copy, Clone, Default)]
pub struct PopoverExpanded(pub bool);

/// Popover's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct PopoverStyle {
    /// content surface fill
    pub background: Color,
}

#[derive(Component, Clone)]
pub(crate) struct PopoverConfig {
    trigger: SlotFn,
    content: SlotFn,
    placement: PopoverPlacement,
    /// how far the content reaches from the trigger -- no general "size to fit arbitrary
    /// children" mechanism exists in this framework (only `Text`/`TextInput` have a real
    /// auto-size path, via font-shaping measurement), so this is an explicit, unit-agnostic
    /// author choice (`200.px()`, `40.pct()`, whatever fits their content), not a guess
    /// standing in for a value that's actually derivable. The cross-axis (width for
    /// Below/Above, height for Left/Right) is never asked for -- it's always the trigger's
    /// own real size, same relationship Dropdown's list already uses.
    extent: LocationValue,
}

/// Emitted at the popover root the moment it opens.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct PopoverOpened {}
/// Emitted at the popover root the moment it closes (click-away or programmatic).
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct PopoverClosed {}

/// Private child registry -- the click-away and rebuild paths need these ids.
#[derive(Component, Copy, Clone)]
pub(crate) struct PopoverHandle {
    trigger: Entity,
    content: Option<Entity>,
}

pub struct PopoverSprout {
    leaf: LeafSprout,
    trigger: Option<SlotFn>,
    content: Option<SlotFn>,
    style: PopoverStyle,
    placement: PopoverPlacement,
    extent: Option<LocationValue>,
}
impl PopoverSprout {
    /// What's always visible -- the tap target. Built once at spawn, never torn down.
    pub fn trigger(
        mut self,
        f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static,
    ) -> Self {
        self.trigger = Some(Arc::new(f));
        self
    }
    /// What shows while expanded -- built fresh on open, removed on close.
    pub fn content(
        mut self,
        f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static,
    ) -> Self {
        self.content = Some(Arc::new(f));
        self
    }
    pub fn colors(mut self, background: Color) -> Self {
        self.style = PopoverStyle { background };
        self
    }
    /// Which side of the trigger to open on. Defaults to `Below`.
    pub fn placement(mut self, p: PopoverPlacement) -> Self {
        self.placement = p;
        self
    }
    /// How far the content reaches from the trigger -- any unit this framework's own
    /// `Location` vocabulary supports (`200.px()`, `40.pct()`, ...), not just pixels.
    /// Required: there's no sensible universal default for arbitrary content's own size.
    pub fn extent(mut self, value: LocationValue) -> Self {
        self.extent = Some(value);
        self
    }
}
impl Sprout for PopoverSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Popover {},
            PopoverConfig {
                trigger: self.trigger.expect("Popover::trigger(..) is required"),
                content: self.content.expect("Popover::content(..) is required"),
                placement: self.placement,
                extent: self.extent.expect("Popover::extent(..) is required"),
            },
            PopoverExpanded(false),
            self.style,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // trigger slot: passes interaction through to `this`'s own click handler, mirroring
        // Dropdown's trigger panel/text exactly.
        let trigger_slot = tree.branch(
            this,
            Leaf::sprout()
                .at(Location::new().xs(
                    0.pct().as_left().with(100.pct().as_right()),
                    0.pct().as_top().with(100.pct().as_bottom()),
                ))
                .elevate(Elevation::up(1))
                .with((
                    Grid::default(),
                    Root(this),
                    InteractionPropagation::pass_through(),
                )),
        );
        tree.write_to(
            this,
            PopoverHandle {
                trigger: trigger_slot,
                content: None,
            },
        );
        // trigger content is built once from config -- config never changes after spawn
        // (there's no `.trigger(..)`-rewrite door), so `react`'s single first-fire is enough.
        tree.react::<PopoverConfig, _>(
            this,
            move |trigger: Trigger<Insert, PopoverConfig>,
                  configs: Query<&PopoverConfig>,
                  handles: Query<&PopoverHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let handle = *handles.get(e).unwrap();
                let cfg = configs.get(e).unwrap().clone();
                (cfg.trigger)(&mut tree, handle.trigger);
            },
        );
        tree.on_click(
            this,
            |trigger: Trigger<OnClick>, expanded: Query<&PopoverExpanded>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.write_to(e, PopoverExpanded(!expanded.get(e).unwrap().0));
            },
        );
        // structure: the content surface exists only while expanded -- rebuilt fresh each
        // time it opens (matches Modal's content lifecycle, not Dropdown's list, since
        // arbitrary author content has no cheaper "just recolor it" patch path in general).
        tree.react::<PopoverExpanded, _>(
            this,
            move |trigger: Trigger<Insert, PopoverExpanded>,
                  expanded: Query<&PopoverExpanded>,
                  configs: Query<&PopoverConfig>,
                  styles: Query<&PopoverStyle>,
                  mut handles: Query<&mut PopoverHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let open = expanded.get(e).unwrap().0;
                let cfg = configs.get(e).unwrap().clone();
                let style = *styles.get(e).unwrap();
                let mut handle = handles.get_mut(e).unwrap();
                if let Some(existing) = handle.content.take() {
                    tree.remove(existing);
                }
                if open {
                    // top-level (not branched under `e`): `pct()` in `.extent(..)` needs to
                    // resolve against the viewport, not this popover's own small trigger-sized
                    // box -- same reason Dropdown's option list is `tree.leaf(..)` too.
                    let surface = tree.leaf(
                        Panel::new()
                            .color(style.background)
                            .at(cfg.placement.location(cfg.extent))
                            .elevate(Elevation::abs(90))
                            .with((Anchor::new(e), Grid::default(), Root(e))),
                    );
                    (cfg.content)(&mut tree, surface);
                    handle.content = Some(surface);
                    tree.trigger_targets(PopoverOpened::new(), e);
                } else {
                    tree.trigger_targets(PopoverClosed::new(), e);
                }
            },
        );
        // click-away: collapse when focus leaves the popover -- unless it landed on the
        // popover's own content surface (content carries Root(this) the same way Dropdown's
        // option rows do).
        tree.subscribe(
            this,
            move |trigger: Trigger<Unfocused>,
                  current_interaction: Res<CurrentInteraction>,
                  roots: Query<&Root>,
                  handles: Query<&PopoverHandle>,
                  expanded: Query<&PopoverExpanded>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                if !expanded.get(e).unwrap().0 {
                    return;
                }
                if let Some(f) = current_interaction.focused {
                    let handle = handles.get(e).unwrap();
                    if f == e || Some(f) == handle.content || Root::resolve(f, &roots) == e {
                        return;
                    }
                }
                tree.write_to(e, PopoverExpanded(false));
            },
        );
    }
}
