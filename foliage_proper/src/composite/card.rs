use crate::Trigger;
use crate::composite::SlotFn;
use crate::{
    Color, Component, EcsExtension, Elevation, Entity, Grid, GridExt, Leaf, LeafSprout, Location,
    Panel, Sprout, Tree,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use std::sync::Arc;

/// A card is one entity -- itself the backdrop panel -- holding up to three regions of
/// author content via the slot convention: `main` (required, the top two-thirds), and an
/// optional `header`/`desc` pair sharing the bottom third as their own two-row grouping
/// (header above desc).
///
/// Positioned via `.at(..)`. To rest one card over another entity's rect, use the same
/// general mechanism any entity uses to position itself relative to another --
/// `.with(Anchor::new(target))` plus an `anchor()`-relative `Location` (see
/// [`Dropdown`](crate::Dropdown)'s option surface for this exact pattern).
///
/// Has no close/remove mechanism of its own -- `tree.remove(card)` is the same general
/// removal every entity uses (see [`Remove`](crate::composite::mod)'s cascade through
/// `Stem` children).
#[derive(Component, Copy, Clone)]
pub struct Card {}
impl Card {
    pub fn new() -> CardSprout {
        CardSprout {
            leaf: LeafSprout::default(),
            main: None,
            header: None,
            desc: None,
            style: CardStyle::default(),
        }
    }
}

/// Card's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct CardStyle {
    pub backdrop: Color,
}

#[derive(Component, Clone)]
pub(crate) struct CardConfig {
    main: SlotFn,
    header: Option<SlotFn>,
    desc: Option<SlotFn>,
}

/// Private child registry: the rebuild path (a config rewrite) needs these ids to tear the
/// right entities down before spawning fresh ones.
#[derive(Component, Copy, Clone)]
pub(crate) struct CardHandle {
    main_slot: Entity,
    header_slot: Option<Entity>,
    desc_slot: Option<Entity>,
}

pub struct CardSprout {
    leaf: LeafSprout,
    main: Option<SlotFn>,
    header: Option<SlotFn>,
    desc: Option<SlotFn>,
    style: CardStyle,
}
impl CardSprout {
    /// The card's main region -- the top two-thirds. Required.
    pub fn main(mut self, f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static) -> Self {
        self.main = Some(Arc::new(f));
        self
    }
    /// The header row within the bottom third. Optional -- skipping it spawns no header
    /// slot at all, not a defunct empty one.
    pub fn header(
        mut self,
        f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static,
    ) -> Self {
        self.header = Some(Arc::new(f));
        self
    }
    /// The description row within the bottom third, below the header. Optional, same
    /// contract as [`Self::header`].
    pub fn desc(mut self, f: impl Fn(&mut Tree, Entity) -> Entity + Send + Sync + 'static) -> Self {
        self.desc = Some(Arc::new(f));
        self
    }
    pub fn colors(mut self, backdrop: Color) -> Self {
        self.style = CardStyle { backdrop };
        self
    }
}
impl Sprout for CardSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Card {},
            CardConfig {
                main: self.main.expect("CardSprout::main(..) is required"),
                header: self.header,
                desc: self.desc,
            },
            self.style,
            Panel::default(),
            Grid::new(1.col(), 3.row()),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        tree.react::<CardConfig, _>(
            this,
            move |trigger: Trigger<Insert, CardConfig>,
                  configs: Query<&CardConfig>,
                  styles: Query<&CardStyle>,
                  handles: Query<&CardHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let cfg = configs.get(e).unwrap().clone();
                let style = *styles.get(e).unwrap();
                // config rewrite = fresh content, per the slot convention
                if let Ok(prior) = handles.get(e) {
                    tree.remove(prior.main_slot);
                    if let Some(s) = prior.header_slot {
                        tree.remove(s);
                    }
                    if let Some(s) = prior.desc_slot {
                        tree.remove(s);
                    }
                }
                tree.write_to(e, style.backdrop);

                // main: the top two-thirds -- rows 1-2 of the root's own 3-row grid.
                let main_slot = tree.branch(
                    e,
                    Leaf::sprout()
                        .at(Location::new().xs(
                            0.pct().as_left().with(100.pct().as_right()),
                            1.row().as_top().with(2.row().as_bottom()),
                        ))
                        .elevate(Elevation::up(1))
                        .with(Grid::default()),
                );
                (cfg.main)(&mut tree, main_slot);

                // header/desc: their own two-row grouping inside the bottom third (row 3).
                let (header_slot, desc_slot) = if cfg.header.is_some() || cfg.desc.is_some() {
                    let bottom_third = tree.branch(
                        e,
                        Leaf::sprout()
                            .at(Location::new().xs(
                                0.pct().as_left().with(100.pct().as_right()),
                                3.row().as_top().with(3.row().as_bottom()),
                            ))
                            .elevate(Elevation::up(1))
                            .with(Grid::new(1.col(), 2.row())),
                    );
                    let header_slot = cfg.header.as_ref().map(|f| {
                        let slot = tree.branch(
                            bottom_third,
                            Leaf::sprout()
                                .at(Location::new().xs(
                                    0.pct().as_left().with(100.pct().as_right()),
                                    1.row().as_top().with(1.row().as_bottom()),
                                ))
                                .elevate(Elevation::up(1))
                                .with(Grid::default()),
                        );
                        f(&mut tree, slot);
                        slot
                    });
                    let desc_slot = cfg.desc.as_ref().map(|f| {
                        let slot = tree.branch(
                            bottom_third,
                            Leaf::sprout()
                                .at(Location::new().xs(
                                    0.pct().as_left().with(100.pct().as_right()),
                                    2.row().as_top().with(2.row().as_bottom()),
                                ))
                                .elevate(Elevation::up(1))
                                .with(Grid::default()),
                        );
                        f(&mut tree, slot);
                        slot
                    });
                    (header_slot, desc_slot)
                } else {
                    (None, None)
                };

                tree.write_to(
                    e,
                    CardHandle {
                        main_slot,
                        header_slot,
                        desc_slot,
                    },
                );
            },
        );
        // later style pokes; the config reaction handles first application (its handle may
        // not be visible to this reaction's own first fire in the same command batch).
        tree.react::<CardStyle, _>(
            this,
            move |trigger: Trigger<Insert, CardStyle>,
                  styles: Query<&CardStyle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let style = *styles.get(e).unwrap();
                tree.write_to(e, style.backdrop);
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::area::Area;
    use crate::ginkgo::viewport::ViewportHandle;
    use crate::{Foliage, Section};
    use std::sync::Mutex;

    // A root-level `Card` positions itself with `.pct()` (unanchored) against the
    // viewport -- `tests/card.rs` (the external integration-test crate) has no way to set
    // this up, since `ginkgo` is crate-private and not re-exported. These geometry checks
    // live here instead, where `ViewportHandle` is directly reachable.
    fn set_realistic_viewport(foliage: &mut Foliage) {
        foliage
            .world
            .insert_resource(ViewportHandle::new(Area::logical((1000.0, 1000.0))));
    }

    fn leaf_child(tree: &mut Tree, slot: Entity) -> Entity {
        tree.branch(
            slot,
            crate::Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(1)),
        )
    }

    fn children_of(foliage: &mut Foliage, parent: Entity) -> Vec<Entity> {
        let mut q = foliage.world.query::<(Entity, &crate::Stem)>();
        q.iter(&foliage.world)
            .filter(|(_, stem)| stem.id == Some(parent))
            .map(|(e, _)| e)
            .collect()
    }

    fn section_of(foliage: &mut Foliage, entity: Entity) -> Section<crate::Logical> {
        *foliage
            .world
            .get::<Section<crate::Logical>>(entity)
            .expect("Section<Logical> is required on every Leaf")
    }

    #[test]
    fn main_occupies_the_top_two_thirds_of_the_card() {
        let mut foliage = Foliage::new();
        set_realistic_viewport(&mut foliage);
        let card = foliage.world.leaf(
            Card::new()
                .main(leaf_child)
                .at(Location::new().xs(
                    0.px().as_left().with(300.px().as_width()),
                    0.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(50)),
        );
        foliage.world.flush();

        let card_section = section_of(&mut foliage, card);
        assert_ne!(
            card_section.height(),
            0.0,
            "sanity: the card itself must have resolved to a real size"
        );
        let main_slot = children_of(&mut foliage, card)[0];
        let main_section = section_of(&mut foliage, main_slot);

        assert_eq!(main_section.top(), card_section.top());
        assert_eq!(main_section.left(), card_section.left());
        assert_eq!(main_section.width(), card_section.width());
        assert!(
            (main_section.height() - card_section.height() * (2.0 / 3.0)).abs() < 0.5,
            "main should span the top two-thirds: got {} of {}",
            main_section.height(),
            card_section.height()
        );
    }

    #[test]
    fn header_sits_above_desc_within_the_bottom_third() {
        let mut foliage = Foliage::new();
        set_realistic_viewport(&mut foliage);

        // records the exact slot Entity each closure is actually invoked with -- direct
        // knowledge of which physical entity is header vs desc, not a guess based on
        // spawn order or `Branch::ids`' `HashSet` iteration order.
        let header_slot: std::sync::Arc<Mutex<Option<Entity>>> = Default::default();
        let desc_slot: std::sync::Arc<Mutex<Option<Entity>>> = Default::default();
        let (h, d) = (header_slot.clone(), desc_slot.clone());
        let card = foliage.world.leaf(
            Card::new()
                .main(leaf_child)
                .header(move |tree, slot| {
                    *h.lock().unwrap() = Some(slot);
                    leaf_child(tree, slot)
                })
                .desc(move |tree, slot| {
                    *d.lock().unwrap() = Some(slot);
                    leaf_child(tree, slot)
                })
                .at(Location::new().xs(
                    0.px().as_left().with(300.px().as_width()),
                    0.px().as_top().with(300.px().as_height()),
                ))
                .elevate(Elevation::up(50)),
        );
        foliage.world.flush();

        let header_slot = header_slot
            .lock()
            .unwrap()
            .expect("header closure should have been called");
        let desc_slot = desc_slot
            .lock()
            .unwrap()
            .expect("desc closure should have been called");
        assert_ne!(header_slot, desc_slot);

        let card_section = section_of(&mut foliage, card);
        let header_section = section_of(&mut foliage, header_slot);
        let desc_section = section_of(&mut foliage, desc_slot);

        let bottom_third_height = header_section.height() + desc_section.height();
        assert!(
            (bottom_third_height - card_section.height() / 3.0).abs() < 0.5,
            "header + desc together should span the bottom third: got {} of {}",
            bottom_third_height,
            card_section.height()
        );
        assert!(
            header_section.top() < desc_section.top(),
            "header should sit above desc within the bottom third"
        );
        assert!(
            (header_section.height() - desc_section.height()).abs() < 0.5,
            "header and desc should each take half the bottom third"
        );
    }
}
