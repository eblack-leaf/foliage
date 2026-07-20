use crate::Trigger;
use crate::composite::{IndexedSlotFn, Root};
use crate::{
    Component, EcsExtension, Elevation, Entity, Grid, GridExt, InteractionListener, Leaf,
    LeafSprout, Location, Sprout, Tree, View,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use std::sync::Arc;

/// A list is one entity: components in ([`ListItems`], [`ListLayout`]), rows out -- as
/// author content, via the slot convention (see [`crate::composite::IndexedSlotFn`]). The
/// list owns the scroll viewport (`View` + listener = wheel/touch scrolling, ancestor
/// clipping for free) and the uniform row layout; what a row IS -- visuals, interaction,
/// selection semantics -- belongs entirely to the author's closure. Each slot carries
/// `Root(list)` so row handlers can route back via [`Root::resolve`]. No events of its own.
///
/// Rewriting [`ListItems`] via `write_to` is the re-render API: prior rows are torn down
/// and the closure runs again per index. No virtualization -- `count` is expected O(dozens).
#[derive(Component, Copy, Clone)]
pub struct List {}
impl List {
    pub fn new() -> ListSprout {
        ListSprout::default()
    }
}

/// List's row-content config: the count and the per-index builder, rewritten as one unit:
/// `tree.write_to(list, ListItems::new(count, |tree, slot, i| { .. }))`.
#[derive(Component, Clone)]
pub struct ListItems {
    pub count: usize,
    pub(crate) builder: IndexedSlotFn,
}
impl ListItems {
    pub fn new(
        count: usize,
        builder: impl Fn(&mut Tree, Entity, usize) + Send + Sync + 'static,
    ) -> Self {
        Self {
            count,
            builder: Arc::new(builder),
        }
    }
}

/// List's OWN layout vocabulary -- geometry, not content, so poking row size doesn't
/// require re-stating the builder.
#[derive(Component, Copy, Clone)]
pub struct ListLayout {
    /// uniform row height in px
    pub row_height: i32,
    /// vertical gap between rows in px
    pub gap: i32,
}
impl Default for ListLayout {
    fn default() -> Self {
        Self {
            row_height: 48,
            gap: 8,
        }
    }
}

pub struct ListSprout {
    leaf: LeafSprout,
    items: Option<ListItems>,
    layout: ListLayout,
}
impl Default for ListSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            items: None,
            layout: ListLayout::default(),
        }
    }
}
impl ListSprout {
    /// Takes the same [`ListItems`] a rebuild writes -- spawn and re-render share one
    /// construction form: `.items(ListItems::new(n, |tree, slot, i| ..))` here,
    /// `tree.write_to(list, ListItems::new(..))` later.
    pub fn items(mut self, items: ListItems) -> Self {
        self.items = Some(items);
        self
    }
    pub fn row_height(mut self, h: i32) -> Self {
        self.layout.row_height = h;
        self
    }
    pub fn gap(mut self, g: i32) -> Self {
        self.layout.gap = g;
        self
    }
}
impl Sprout for ListSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            List {},
            self.items.expect("List::items(..) is required"),
            self.layout,
            View::new(),
            InteractionListener::new(),
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // no static skeleton: everything is data-dependent (row count, row size), so it all
        // lands via the reaction's first fire. Captured Vec tracks the slots for teardown --
        // the ProjectCard-image pattern; removal cascades through each slot's Stem children,
        // so the author's whole row goes with it.
        let mut slots: Vec<Entity> = Vec::new();
        tree.react_any::<(ListItems, ListLayout), _>(
            this,
            move |trigger: Trigger<Insert, (ListItems, ListLayout)>,
                  items: Query<&ListItems>,
                  layouts: Query<&ListLayout>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let items = items.get(e).unwrap().clone();
                let layout = *layouts.get(e).unwrap();
                for slot in slots.drain(..) {
                    tree.remove(slot);
                }
                tree.write_to(
                    e,
                    Grid::new(
                        1.col().gap(layout.gap),
                        layout.row_height.px().gap(layout.gap),
                    ),
                );
                for i in 0..items.count {
                    let slot = tree.branch(
                        e,
                        Leaf::sprout()
                            .at(Location::new().xs(
                                1.col().as_left().with(1.col().as_right()),
                                (i + 1).row().as_top().with((i + 1).row().as_bottom()),
                            ))
                            .elevate(Elevation::up(1))
                            .with((Grid::default(), Root(e))),
                    );
                    (items.builder)(&mut tree, slot, i);
                    slots.push(slot);
                }
            },
        );
    }
}
