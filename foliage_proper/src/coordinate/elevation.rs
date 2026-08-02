use crate::AsTree;
use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::ash::clip::ClipToViewport;
use crate::{Animate, Attachment, Children, Foliage, Parent, Resolve, Tree};
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
/// The depth value the GPU actually sorts on, derived from `StackKey` order (engine-internal)
/// rather than from [`Elevation`] arithmetic. Read-only; write [`Elevation`] to change it.
pub struct ResolvedElevation(pub(crate) f32);
impl ResolvedElevation {
    /// The raw depth scalar.
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
    /// TODO: `resets` is driven by `ClipToViewport`, which conflates "escape ancestor clips"
    /// with "become a front-tier overlay" -- see that type's own TODO. When it splits into
    /// `ClipTo`, this tier reset needs its own opt-in (an `Overlay` marker) rather than
    /// riding along with a clipping choice, or everything that only wanted to overhang its
    /// parent silently floats in front of the whole page.
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
/// Where an entity sits in draw order, relative to its parent or pinned to an absolute
/// layer.
///
/// Lower draws in front. [`up`](Elevation::up)/[`down`](Elevation::down) are relative to
/// the parent and compose down the tree, which is what makes a subtree movable in depth as
/// a unit; [`abs`](Elevation::abs) leaves the hierarchy and names a fixed layer, for
/// something that must float above everything regardless of where it is parented.
///
/// Prefer relative: absolute values from unrelated composites can collide at the same
/// number with nothing to arbitrate them.
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
    /// A fixed layer, ignoring the parent's own elevation. Higher `e` draws in front.
    pub fn abs(e: i32) -> Self {
        Self {
            amount: 100f32 - e as f32,
            absolute: true,
        }
    }
    /// `u` steps in front of the parent.
    pub fn up(u: i32) -> Self {
        Self {
            amount: u as f32 * -1f32,
            absolute: false,
        }
    }
    /// `d` steps behind the parent.
    pub fn down(d: i32) -> Self {
        Self {
            amount: d as f32,
            absolute: false,
        }
    }
    fn stem_insert(trigger: Trigger<Insert, Parent>, mut tree: Tree) {
        tree.send_to(Resolve::<Elevation>::new(), trigger.event_target());
    }
    fn update(
        trigger: Trigger<Resolve<Elevation>>,
        mut tree: Tree,
        stack_keys: Query<&StackKey>,
        clip_to_viewport: Query<&ClipToViewport>,
        elevation: Query<&Elevation>,
        stem: Query<&Parent>,
        branch: Query<&Children>,
    ) {
        let this = trigger.event_target();
        if stem.get(this).ok().is_none() || branch.get(this).ok().is_none() {
            return;
        }
        // This computes only `StackKey` -- the tree-structured *ordering* truth. It does NOT
        // write `ResolvedElevation`: that (the GPU depth scalar) is owned solely by
        // `ash::assign_elevations`, which derives it from `StackKey` order via a
        // gapped/fractional-index scheme. Writing it from here as well would fight that:
        // the additive value can fall outside the GPU's depth range on a deep `up(n)`
        // chain, and since `assign_elevations` only re-derives on `StackKey` *change*, a
        // later re-run here would overwrite the good rank with the bad additive one and
        // never reclaim it.
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
        tree.write_to(this, stack_key);
        for dep in branch.get(this).unwrap().ids.clone() {
            if let Some(elev) = elevation.get(dep).copied().ok() {
                tree.write_to(dep, elev);
            }
        }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.tree().send_to(Resolve::<Elevation>::new(), this);
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
    /// Wraps a raw depth scalar. Written by `ash::assign_elevations` from `StackKey`
    /// order -- set [`Elevation`] instead.
    pub fn new(l: f32) -> Self {
        Self(l)
    }
}
