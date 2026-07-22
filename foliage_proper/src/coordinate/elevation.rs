use crate::EcsExtension;
use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::ash::clip::ClipToViewport;
use crate::{Animate, Attachment, Branch, Foliage, Stem, Tree, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Component;
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use bytemuck::{Pod, Zeroable};
use std::fmt::Display;
use std::ops::{Add, Sub};

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Pod, Zeroable, Component, Debug)]
pub struct ResolvedElevation(pub(crate) f32);
impl ResolvedElevation {
    pub fn value(&self) -> f32 {
        self.0
    }
    /// The one canonical front-to-back comparison (smaller raw value = more in front,
    /// `Greater`) -- previously hand-reimplemented independently in three places
    /// (`PartialOrd` here, `ash/mod.rs`'s node sort, `ash/instance.rs`'s
    /// `InstanceCoordinator::sort`), which had to be kept in sync by hand. All three now call
    /// this instead.
    pub(crate) fn front_to_back(&self, other: &Self) -> std::cmp::Ordering {
        other.0.total_cmp(&self.0)
    }
}
/// A packed, fixed-width, tree-structured ordering key -- one small field per ancestor
/// level (root-most = most-significant byte, this entity's own field = least-significant
/// *used* byte), replacing the old flat `ResolvedElevation`-only comparison for both render
/// order and click-priority. Comparing two `StackKey`s (`Ord`, derived purely from `.key`)
/// gives exactly CSS-stacking-context semantics: two entities only diverge starting at their
/// first differing ancestor level, so a composite author's own local sibling ordering can
/// never be numerically overridden by unrelated content elsewhere in the tree, no matter how
/// deep either one is nested. `Elevation::abs`/`up`/`down` still just decide *one field's*
/// value (raw vs. added to the parent's own field) -- entirely orthogonal to whether the
/// *prefix* resets, which is driven only by `ClipToViewport` (see `update`, below).
///
/// Each byte is a *biased* i8 (`raw + 128`, so -128..127 maps to 0..255) rather than a plain
/// two's-complement byte, so plain unsigned-byte comparison (which is what comparing the
/// packed `u128` as a whole does) still orders negative/positive local amounts correctly.
/// `NEUTRAL_BYTE` (0x80, i.e. biased zero) is the baseline every not-yet-assigned level
/// implicitly holds -- both a stem-less root's fallback "parent" and what `ClipToViewport`
/// resets its prefix to before writing its own field.
///
/// 16 levels comfortably covers the deepest real nesting seen in this codebase (~6-8, e.g.
/// a Dropdown's option surface nested inside a real page's modal/app containers); anything
/// deeper saturates at the last slot -- still correctly ordered up to that depth, just no
/// longer distinguishable *beyond* it, a documented limit rather than a silent one.
#[derive(Copy, Clone, Debug, Component)]
pub(crate) struct StackKey {
    key: u128,
    depth: u8,
}
impl StackKey {
    const LEVELS: u32 = 16;
    const BITS_PER_LEVEL: u32 = 8;
    const NEUTRAL_BYTE: u8 = 0x80;
    /// The most-significant byte (byte 0) is a global *stacking tier*, not a structural
    /// elevation field -- every entity's structural per-level fields live in bytes 1..15.
    /// `NEUTRAL_TIER` is ordinary content; `FRONT_TIER` (smaller = more in front) is a
    /// `ClipToViewport` overlay subtree, which must float in front of *all* ordinary content
    /// regardless of anyone's `abs()`/`up()` values. Without a dedicated tier, a reset overlay
    /// competed at byte 0 with ordinary chrome using a forward `abs()` (a modal at `abs(50)`, a
    /// button at `abs(95)`) and lost -- rendering behind, and losing clicks to, all of it.
    const NEUTRAL_TIER: u8 = 0x80;
    const FRONT_TIER: u8 = 0x00;
    /// sentinel meaning "no parent" (a stem-less root, or a `ClipToViewport` overlay root) --
    /// its own first structural field lands at depth 0 (byte 1, just under the tier byte).
    const NO_PARENT: u8 = u8::MAX;

    fn baseline(tier: u8) -> u128 {
        let mut bytes = [Self::NEUTRAL_BYTE; 16];
        bytes[0] = tier;
        u128::from_be_bytes(bytes)
    }
    /// This entity's own `StackKey`, given the parent's own already-resolved `StackKey` (or
    /// `None` for a stem-less root) and whether this entity begins a fresh overlay subtree
    /// (`ClipToViewport`-marked). `depth` is *structural* depth (0 = a subtree root); its
    /// field occupies byte `1 + depth` (byte 0 is the tier), so ordinary structural nesting
    /// never touches the tier -- descendants inherit the parent's whole key (tier included),
    /// keeping an overlay's children in the overlay tier.
    fn compute(parent: Option<StackKey>, resets: bool, amount: f32) -> StackKey {
        let (base_key, parent_depth) = if resets {
            // ClipToViewport: fresh overlay subtree root in the FRONT tier.
            (Self::baseline(Self::FRONT_TIER), Self::NO_PARENT)
        } else if let Some(p) = parent {
            // inherit the parent's tier byte *and* structural prefix.
            (p.key, p.depth)
        } else {
            // ordinary stem-less root.
            (Self::baseline(Self::NEUTRAL_TIER), Self::NO_PARENT)
        };
        let depth = if parent_depth == Self::NO_PARENT {
            0
        } else {
            // structural fields occupy bytes 1..15, so the deepest usable structural depth is
            // LEVELS - 2 (byte 15); anything deeper saturates there.
            ((parent_depth as u32 + 1).min(Self::LEVELS - 2)) as u8
        };
        let byte_index = 1 + depth as u32;
        let biased = ((amount.round().clamp(-128.0, 127.0)) as i32 + 128) as u8;
        let shift = (Self::LEVELS - 1 - byte_index) * Self::BITS_PER_LEVEL;
        let mask = !(0xFFu128 << shift);
        let key = (base_key & mask) | ((biased as u128) << shift);
        StackKey { key, depth }
    }
}
impl Default for StackKey {
    fn default() -> Self {
        StackKey {
            key: Self::baseline(Self::NEUTRAL_TIER),
            depth: Self::NO_PARENT,
        }
    }
}
impl PartialEq for StackKey {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for StackKey {}
impl PartialOrd for StackKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for StackKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // smaller raw key = more in front, matching `ResolvedElevation`'s existing inverted
        // convention (so `up(n)` continues to mean "more in front" everywhere it's compared).
        other.key.cmp(&self.key)
    }
}
impl Attachment for Elevation {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Elevation::update);
        foliage.define(Elevation::stem_insert);
        foliage.enable_animation::<Self>();
    }
}
impl Display for ResolvedElevation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}
impl PartialOrd for ResolvedElevation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.front_to_back(other))
    }
}
#[derive(Copy, Clone, PartialEq, PartialOrd, Component, Debug)]
#[require(ResolvedElevation, StackKey)]
#[component(on_insert = Self::on_insert)]
pub struct Elevation {
    pub amount: f32,
    pub(crate) absolute: bool,
}
impl Default for Elevation {
    fn default() -> Self {
        Self::abs(0)
    }
}
impl Elevation {
    pub fn abs(e: i32) -> Self {
        Self {
            amount: 100f32 - e as f32,
            absolute: true,
        }
    }
    pub fn up(u: i32) -> Self {
        Self {
            amount: u as f32 * -1f32,
            absolute: false,
        }
    }
    pub fn down(d: i32) -> Self {
        Self {
            amount: d as f32,
            absolute: false,
        }
    }
    fn stem_insert(trigger: Trigger<Insert, Stem>, mut tree: Tree) {
        tree.trigger_targets(Update::<Elevation>::new(), trigger.event_target());
    }
    fn update(
        trigger: Trigger<Update<Elevation>>,
        mut tree: Tree,
        stack_keys: Query<&StackKey>,
        clip_to_viewport: Query<&ClipToViewport>,
        elevation: Query<&Elevation>,
        stem: Query<&Stem>,
        branch: Query<&Branch>,
    ) {
        let this = trigger.event_target();
        if stem.get(this).ok().is_none() || branch.get(this).ok().is_none() {
            return;
        }
        // This computes only `StackKey` -- the tree-structured *ordering* truth. It does NOT
        // write `ResolvedElevation`: that (the GPU depth scalar) is owned solely by
        // `ash::assign_elevations`, which derives it from `StackKey` order via a
        // gapped/fractional-index scheme. Writing it here too used to fight that: the additive
        // value could resolve outside the GPU's depth range (a deep `up(n)` chain), and since
        // `assign_elevations` only re-derives on `StackKey` *change* (position-independent),
        // any later re-run of this system would clobber the good rank value with the bad
        // additive one and it would never get reclaimed -- an overlay flashing in for one
        // frame then depth-culling away.
        //
        // `StackKey`: a `ClipToViewport`-marked entity resets its prefix (fresh top-level
        // baseline) instead of inheriting the parent's key; every other entity inherits it
        // normally. Each level's field is this entity's own local "how far forward" amount --
        // for `up`/`down`, `elev.amount` (a small relative delta, e.g. -1/+1) already *is*
        // that. `abs(e)`'s stored `amount` (`100.0 - e`) is on the old flat *global* scale
        // (0..100), so it's inverted back to `-e` (`elev.amount - 100.0`) to sit on the same
        // per-level scale as `up`/`down` -- an `abs(90)` sibling then correctly compares more
        // in front than a plain `up(1)` sibling, and `abs(0)` (`Elevation::default()`) maps to
        // field `0`, exactly `StackKey`'s `NEUTRAL_BYTE` baseline.
        let parent = stem.get(this).unwrap().id;
        let elev = elevation.get(this).unwrap();
        let parent_key = parent.and_then(|id| stack_keys.get(id).copied().ok());
        let resets = clip_to_viewport.get(this).is_ok();
        let field = if elev.absolute {
            elev.amount - 100.0
        } else {
            elev.amount
        };
        let stack_key = StackKey::compute(parent_key, resets, field);
        tracing::trace!(entity = ?this, ?stack_key, "elevation: computed stack key");
        tree.entity(this).insert(stack_key);
        for dep in branch.get(this).unwrap().ids.clone() {
            if let Some(elev) = elevation.get(dep).copied().ok() {
                tree.entity(dep).insert(elev);
            }
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .trigger_targets(Update::<Elevation>::new(), this);
    }
}
impl Animate for Elevation {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        Interpolations::new().with(start.amount, end.amount)
    }
    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(e) = interpolations.read(0) {
            self.amount = e;
        }
    }
}
impl Add for ResolvedElevation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}
impl Sub for ResolvedElevation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}
impl ResolvedElevation {
    pub fn new(l: f32) -> Self {
        Self(l)
    }
}

#[cfg(test)]
mod stack_key_tests {
    use super::*;
    use crate::{EcsExtension, Foliage, Grid, Leaf, Location, Sprout};

    fn key_of(foliage: &mut Foliage, entity: Entity) -> StackKey {
        *foliage.world.get::<StackKey>(entity).unwrap()
    }

    /// mask covering every field *except* this entity's own structural field -- for asserting
    /// two keys share a prefix everywhere but the one level that's expected to legitimately
    /// differ. The field lives at byte `1 + depth` (byte 0 is the tier).
    fn mask_excluding(depth: u8) -> u128 {
        let byte_index = 1 + depth as u32;
        let shift = (StackKey::LEVELS - 1 - byte_index) * StackKey::BITS_PER_LEVEL;
        !(0xFFu128 << shift)
    }

    #[test]
    fn a_child_inherits_its_parents_key_as_a_prefix() {
        let mut foliage = Foliage::new();
        let parent = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
        let child = foliage.world.branch(
            parent,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        foliage.world.flush();

        let (parent_key, child_key) = (key_of(&mut foliage, parent), key_of(&mut foliage, child));
        // the child's key must differ only in its own field -- the parent's own field carries
        // through unchanged.
        let mask = mask_excluding(child_key.depth);
        assert_eq!(
            parent_key.key & mask,
            child_key.key & mask,
            "child's key should share the parent's prefix exactly, differing only in its own field"
        );
        assert!(
            child_key > parent_key,
            "up(1) should still compare as more in front than its parent under StackKey's Ord"
        );
    }

    #[test]
    fn a_higher_up_amount_resolves_further_in_front_than_a_lower_one() {
        let mut foliage = Foliage::new();
        let parent = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
        let near = foliage.world.branch(
            parent,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        let far = foliage.world.branch(
            parent,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(2)),
        );
        foliage.world.flush();

        assert!(
            key_of(&mut foliage, far) > key_of(&mut foliage, near),
            "up(2) should compare as further in front than up(1) among siblings"
        );
    }

    #[test]
    fn up_resolves_relative_to_the_parents_own_key_not_a_fixed_baseline() {
        // two roots at different absolute elevations -- each child's `up(1)` inherits its own
        // parent's key as a prefix, so the child of the more-in-front root is itself more in
        // front, and the two children's keys genuinely differ (they don't collapse to the same
        // value regardless of context).
        let mut foliage = Foliage::new();
        let low_root = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
        let high_root = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(10)));
        let low_child = foliage.world.branch(
            low_root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        let high_child = foliage.world.branch(
            high_root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        foliage.world.flush();

        assert_ne!(
            key_of(&mut foliage, low_child).key,
            key_of(&mut foliage, high_child).key,
            "both children are up(1), but their parents sit at different absolute elevations, \
             so their keys must differ"
        );
        assert!(
            key_of(&mut foliage, high_child) > key_of(&mut foliage, low_child),
            "the child of the more-in-front root (abs(10)) should itself resolve more in front"
        );
    }

    #[test]
    fn a_chrome_branch_beats_deeply_nested_content_in_a_different_branch_despite_a_raw_elevation_tie() {
        // the real Carousel bug this mechanism exists to prevent: two unrelated branches off
        // the same root, one much deeper than the other, whose *raw* flat elevation sums
        // happen to coincide exactly -- the shallower branch's own local "in front" choice,
        // at the point the two branches actually diverge, must still win regardless of how
        // deep the other branch's own content reaches.
        let mut foliage = Foliage::new();
        let root = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
        let chrome = foliage.world.branch(
            root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(5)),
        );
        let content = foliage.world.branch(
            root,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        let a = foliage.world.branch(
            content,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        let b = foliage
            .world
            .branch(a, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        let c = foliage
            .world
            .branch(b, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        let deep = foliage
            .world
            .branch(c, Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)));
        foliage.world.flush();

        // this is a genuine reproduction of the old flat-model tie, by hand: under the old
        // additive `ResolvedElevation`, root(abs0)=100, chrome(up5)=95, and the deep chain
        // content(up1)=99 -> a=98 -> b=97 -> c=96 -> deep=95 -- chrome and deep both land on
        // exactly 95, an unresolvable coincidence. `StackKey` instead diverges them at `root`
        // (chrome's up(5) branch vs. content's up(1) branch), so chrome wins deterministically.
        assert!(
            key_of(&mut foliage, chrome) > key_of(&mut foliage, deep),
            "chrome's own local ordering (diverging from `deep` at `root`) must win regardless \
             of the old raw-elevation tie"
        );
    }

    #[test]
    fn a_clip_to_viewport_overlay_outranks_chrome_using_a_forward_abs_it_is_nested_under() {
        // the exact real-app bug: a Dropdown surface (`ClipToViewport`, `up(3)`) lives inside
        // a modal spawned at `abs(50)`, with a page `abs(95)` button also around. Resetting the
        // overlay to a *neutral* baseline made its top byte its own `up(3)` field, which lost
        // to the modal's/button's forward `abs()` -- so the overlay rendered behind, and lost
        // clicks to, all that chrome. A reserved FRONT stacking tier must put the overlay in
        // front of every ordinary entity regardless of their `abs()` values.
        let mut foliage = Foliage::new();
        let modal = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(50)));
        let back_button = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(95)));
        let content = foliage.world.branch(
            modal,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        let trigger = foliage.world.branch(
            content,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        let surface = foliage.world.branch(
            trigger,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(3))
                .with(ClipToViewport),
        );
        foliage.world.flush();

        let surface_key = key_of(&mut foliage, surface);
        assert!(
            surface_key > key_of(&mut foliage, modal),
            "the overlay surface must render in front of the modal it's nested inside"
        );
        assert!(
            surface_key > key_of(&mut foliage, content),
            "the overlay surface must render in front of the modal's own content"
        );
        assert!(
            surface_key > key_of(&mut foliage, back_button),
            "the overlay surface must render in front of even a forward `abs(95)` page button"
        );
    }

    #[test]
    fn a_clip_to_viewport_entity_resets_its_prefix_regardless_of_real_nesting_depth() {
        let mut foliage = Foliage::new();
        let root = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::abs(0))
                .with(Grid::default()),
        );
        let deep_trigger = foliage.world.branch(
            root,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        // marked entity, nested several real levels deep -- its own key should be
        // indistinguishable (as far as prefix goes) from one hanging directly off root.
        let surface = foliage.world.branch(
            deep_trigger,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(3))
                .with((ClipToViewport, Grid::default())),
        );
        let surface_child = foliage.world.branch(
            surface,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(1)),
        );
        // control: an ordinary (unmarked) entity at the same real depth as `surface`.
        let ordinary_root = foliage
            .world
            .leaf(Leaf::sprout().at(Location::new()).elevate(Elevation::abs(0)));
        let ordinary_deep = foliage.world.branch(
            ordinary_root,
            Leaf::sprout()
                .at(Location::new())
                .elevate(Elevation::up(1))
                .with(Grid::default()),
        );
        let ordinary_surface_equivalent = foliage.world.branch(
            ordinary_deep,
            Leaf::sprout().at(Location::new()).elevate(Elevation::up(3)),
        );
        foliage.world.flush();

        assert_ne!(
            key_of(&mut foliage, surface).key,
            key_of(&mut foliage, ordinary_surface_equivalent).key,
            "the marked entity's key must NOT carry the real ancestor prefix an equivalent \
             unmarked entity at the same depth would"
        );
        // its own child still nests normally underneath the *reset* prefix, not the real one.
        let child_key = key_of(&mut foliage, surface_child);
        let mask = mask_excluding(child_key.depth);
        assert_eq!(
            key_of(&mut foliage, surface).key & mask,
            child_key.key & mask,
            "surface_child should inherit surface's (reset) prefix, not the real ancestors'"
        );
        assert!(
            key_of(&mut foliage, surface_child) > key_of(&mut foliage, surface),
            "surface_child (up(1) from surface) should still compare more in front than surface itself"
        );
    }
}
