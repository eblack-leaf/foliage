use crate::Trigger;
use crate::composite::SlotFn;
use crate::{
    Anchor, Button, ButtonStyle, Color, Component, EcsExtension, Elevation, Entity, Grid, GridExt,
    IconId, IconValue, Leaf, LeafSprout, Location, OnClick, Opacity, Outline, Panel, Rounding,
    Sprout, Tree,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use std::sync::Arc;

/// A modal is one entity -- itself the full-screen backdrop panel -- holding author content
/// via the slot convention (see [`crate::composite::SlotFn`]). Opens and closes instantly,
/// no animation of its own. Components in ([`ModalStyle`]; the content/anchor/icon config is
/// spawn-time), [`Closed`] out, [`CloseModal`] in -- the close button (present only when
/// `.close_icon(..)` supplies one; the library ships no icons) and programmatic closes share
/// one close path.
///
/// The modal owns its root `Location` (it IS the full-screen overlay rect) -- `.at()` on the
/// sprout is meaningless here. `.elevate()` is the author's: overlays need to know what
/// they're over.
#[derive(Component, Copy, Clone)]
pub struct Modal {}
impl Modal {
    pub fn new() -> ModalSprout {
        ModalSprout {
            leaf: LeafSprout::default(),
            anchor_to: None,
            content: None,
            close_icon: None,
            style: ModalStyle::default(),
        }
    }
}

/// Modal's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct ModalStyle {
    /// the overlay panel's fill
    pub backdrop: Color,
    /// close-button icon color
    pub foreground: Color,
    /// close-button fill
    pub accent: Color,
}

/// Emitted at the modal root the instant it starts closing, immediately before the entity
/// (and everything Stem-parented under it) is removed.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Closed {}

/// Public close command: `tree.trigger_targets(CloseModal::new(), modal)` runs the exact
/// close path the close button does -- same [`Closed`] event, same immediate removal.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct CloseModal {}

#[derive(Component, Clone)]
pub(crate) struct ModalConfig {
    anchor_to: Option<Entity>,
    content: SlotFn,
    close_icon: Option<IconId>,
}

/// Private child registry, TextInput-`Handle`-style: the close observer and later style
/// pokes need these ids, and they're born inside the config reaction.
#[derive(Component, Copy, Clone)]
pub(crate) struct ModalHandle {
    slot: Entity,
    content: Entity,
    terminate: Option<Entity>,
}

fn full_location() -> Location {
    Location::new().xs(
        0.pct().as_left().with(100.pct().as_right()),
        0.pct().as_top().with(100.pct().as_bottom()),
    )
}

pub struct ModalSprout {
    leaf: LeafSprout,
    anchor_to: Option<Entity>,
    content: Option<SlotFn>,
    close_icon: Option<IconId>,
    style: ModalStyle,
}
impl ModalSprout {
    /// The entity the overlay grows out of and shrinks back into (a card's root, say).
    /// Without one, the modal fades/scales from center instead.
    pub fn anchor_to(mut self, e: Entity) -> Self {
        self.anchor_to = Some(e);
        self
    }
    pub fn content(
        mut self,
        f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static,
    ) -> Self {
        self.content = Some(Arc::new(f));
        self
    }
    /// Without one there is no close button -- close via [`CloseModal`].
    pub fn close_icon(mut self, icon: IconId) -> Self {
        self.close_icon = Some(icon);
        self
    }
    pub fn colors(mut self, backdrop: Color, foreground: Color, accent: Color) -> Self {
        self.style = ModalStyle {
            backdrop,
            foreground,
            accent,
        };
        self
    }
}
impl Sprout for ModalSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Modal {},
            ModalConfig {
                anchor_to: self.anchor_to,
                content: self.content.expect("Modal::content(..) is required"),
                close_icon: self.close_icon,
            },
            self.style,
            // the root IS the backdrop: Panel here makes the one public entity the visual
            // overlay itself, so anchor/open/close animations target `this` directly and
            // teardown is a single remove.
            Panel::default(),
            Opacity::new(0.0),
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        tree.react::<ModalConfig, _>(
            this,
            move |trigger: Trigger<Insert, ModalConfig>,
                  configs: Query<&ModalConfig>,
                  styles: Query<&ModalStyle>,
                  handles: Query<&ModalHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let cfg = configs.get(e).unwrap().clone();
                let style = *styles.get(e).unwrap();
                // config rewrite = fresh content, per the slot convention
                if let Ok(prior) = handles.get(e) {
                    tree.remove(prior.slot);
                    if let Some(t) = prior.terminate {
                        tree.remove(t);
                    }
                }
                // Anchor MUST land before Location: Location's own on_insert hook resolves
                // it immediately (synchronously, as this command is applied), and an
                // anchor()-relative Location resolves against whatever Anchor is already on
                // the entity at that moment -- write it after, and the anchor stack lookup
                // fails (no anchor yet), silently falling back to an unanchored origin
                // (top-left) instead of growing from the card.
                if let Some(a) = cfg.anchor_to {
                    tree.write_to(e, Anchor::new(a));
                }
                tree.write_to(e, (full_location(), style.backdrop, Opacity::new(1.0)));
                let slot = tree.branch(
                    e,
                    Leaf::sprout()
                        .at(full_location())
                        .elevate(Elevation::up(1))
                        .with(Grid::default()),
                );
                let content = (cfg.content)(&mut tree, slot);
                let terminate = cfg.close_icon.map(|icon| {
                    let terminate = tree.branch(
                        e,
                        Button::new()
                            .rounding(Rounding::Full)
                            .icon(icon)
                            .colors(style.foreground, style.accent)
                            .at(Location::new().xs(
                                16.px().as_left().with(40.px().as_width()),
                                16.px().as_top().with(40.px().as_height()),
                            ))
                            .elevate(Elevation::up(45)),
                    );
                    tree.on_click(terminate, move |_: Trigger<OnClick>, mut tree: Tree| {
                        tree.trigger_targets(CloseModal::new(), e);
                    });
                    terminate
                });
                tree.write_to(
                    e,
                    ModalHandle {
                        slot,
                        content,
                        terminate,
                    },
                );
            },
        );
        // later style pokes; the config reaction handles first application (its handle may
        // not be visible to this reaction's own first fire in the same command batch).
        tree.react::<ModalStyle, _>(
            this,
            move |trigger: Trigger<Insert, ModalStyle>,
                  styles: Query<&ModalStyle>,
                  handles: Query<&ModalHandle>,
                  configs: Query<&ModalConfig>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let style = *styles.get(e).unwrap();
                tree.write_to(e, style.backdrop);
                if let (Ok(handle), Ok(cfg)) = (handles.get(e), configs.get(e)) {
                    if let (Some(t), Some(icon)) = (handle.terminate, cfg.close_icon) {
                        tree.write_to(
                            t,
                            (
                                IconValue(icon),
                                ButtonStyle {
                                    foreground: style.foreground,
                                    background: style.accent,
                                    outline: Outline::default(),
                                    rounding: Rounding::Full,
                                },
                            ),
                        );
                    }
                }
            },
        );
        // the one close path: close button and programmatic CloseModal both land here.
        // Closes instantly -- no animation. Closed fires before the removal, so an
        // observer can still read the modal's state one last time if it needs to.
        tree.subscribe(
            this,
            move |trigger: Trigger<CloseModal>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.trigger_targets(Closed::new(), e);
                // children (slot content, terminate button) are Stem-parented -- one
                // remove cascades the lot.
                tree.remove(e);
            },
        );
    }
}
