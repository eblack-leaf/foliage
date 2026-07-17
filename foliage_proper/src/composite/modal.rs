use crate::composite::SlotFn;
use crate::Trigger;
use crate::{
    anchor, Anchor, Animation, Button, ButtonStyle, Color, Component, Ease, EcsExtension,
    Elevation, Entity, Grid, IconId, IconValue, LeafSprout, Leaf, Location, GridExt, OnClick,
    OnEnd, Opacity, Outline, Panel, Rounding, Sequence, Sprout, Tree,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use std::sync::Arc;

/// A modal is one entity -- itself the full-screen backdrop panel -- holding author content
/// via the slot convention (see [`crate::composite::SlotFn`]), animating open from an
/// optional anchor entity (or fading/scaling from center without one), and closing back the
/// same way. Components in ([`ModalStyle`]; the content/anchor/icon config is spawn-time),
/// [`Closed`] out, [`CloseModal`] in -- the close button (present only when
/// `.close_icon(..)` supplies one; the library ships no icons) and programmatic closes share
/// one animation path.
///
/// The modal owns its root `Location` (it IS the overlay rect, animated between anchor and
/// full-screen) -- `.at()` on the sprout is meaningless here. `.elevate()` is the author's:
/// overlays need to know what they're over.
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

/// Emitted at the modal root the moment closing BEGINS (not when it finishes) -- the
/// caller's own restore animations are meant to run in parallel with the close animation,
/// not queue behind it.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Closed {}

/// Public close command: `tree.trigger_targets(CloseModal::new(), modal)` runs the exact
/// close path the close button does -- same animation, same [`Closed`] timing, same
/// self-removal.
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

/// The padded near-full-screen rect both open and close animations pass through.
fn padded_location() -> Location {
    Location::new().xs(
        0.pct()
            .as_left()
            .adjust(24)
            .with(100.pct().as_right().adjust(-24))
            .max(450.0),
        0.pct()
            .as_top()
            .adjust(36)
            .with(100.pct().as_bottom().adjust(-36)),
    )
}
fn full_location() -> Location {
    Location::new().xs(
        0.pct().as_left().with(100.pct().as_right()),
        0.pct().as_top().with(100.pct().as_bottom()),
    )
}
/// Where the modal rect starts (open) and returns to (close): the anchor's own rect when
/// one was given, a centered reduced box otherwise.
fn origin_location(anchored: bool) -> Location {
    if anchored {
        Location::new().xs(
            anchor().left().as_left().with(anchor().right().as_right()),
            anchor().top().as_top().with(anchor().bottom().as_bottom()),
        )
    } else {
        Location::new().xs(
            50.pct().as_center_x().with(60.pct().as_width()),
            50.pct().as_center_y().with(60.pct().as_height()),
        )
    }
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
                tree.write_to(e, (origin_location(cfg.anchor_to.is_some()), style.backdrop));
                if let Some(a) = cfg.anchor_to {
                    tree.write_to(e, Anchor::new(a));
                }
                let slot = tree.branch(
                    e,
                    Leaf::sprout()
                        .at(full_location())
                        .elevate(Elevation::up(1))
                        .with((Grid::default(), crate::composite::Root(e))),
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
                Sequence::new(&mut tree)
                    .animate(
                        Animation::new(Opacity::new(1.0))
                            .targeting(e)
                            .start(0)
                            .finish(200),
                    )
                    .animate(
                        Animation::new(padded_location())
                            .targeting(e)
                            .start(0)
                            .finish(750)
                            .eased(Ease::INWARD),
                    )
                    .animate(
                        Animation::new(full_location())
                            .targeting(e)
                            .start(1000)
                            .finish(1500),
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
        tree.subscribe(
            this,
            move |trigger: Trigger<CloseModal>,
                  handles: Query<&ModalHandle>,
                  configs: Query<&ModalConfig>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let handle = *handles.get(e).unwrap();
                let anchored = configs.get(e).unwrap().anchor_to.is_some();
                // content goes immediately -- the shrinking backdrop shouldn't show a
                // squeezed layout reflowing inside it.
                tree.remove(handle.content);
                if let Some(t) = handle.terminate {
                    tree.disable(t);
                }
                // fired now, not in .end() -- the caller's own restore animations run in
                // parallel with this close animation, matching the open choreography.
                tree.trigger_targets(Closed::new(), e);
                let mut seq = Sequence::new(&mut tree).animate(
                    Animation::new(padded_location())
                        .targeting(e)
                        .start(0)
                        .finish(500)
                        .eased(Ease::INWARD),
                );
                if let Some(t) = handle.terminate {
                    seq = seq.animate(
                        Animation::new(Opacity::new(0.0))
                            .targeting(t)
                            .start(0)
                            .finish(500),
                    );
                }
                seq.animate(
                    Animation::new(origin_location(anchored))
                        .targeting(e)
                        .start(750)
                        .finish(1250),
                )
                .animate(
                    Animation::new(Opacity::new(0.0))
                        .targeting(e)
                        .start(1050)
                        .finish(1250),
                )
                .end(move |_: Trigger<OnEnd>, mut tree: Tree| {
                    // children (slot remnants, terminate) are Stem-parented -- one remove
                    // cascades the lot.
                    tree.remove(e);
                });
            },
        );
    }
}
