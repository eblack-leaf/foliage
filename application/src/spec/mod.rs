//! AUTHORING SPEC — intentionally NOT in main.rs's module tree, and it must not be:
//! it is written against library surface that does not exist yet. This is the acceptance
//! test for the composite redesign. If this code reads wrong, the design is wrong, and we
//! fix it here — on paper — before foliage_proper moves.
//!
//! Read order: card_game.rs (the crucible) -> zoo.rs -> screen.rs.
//!
//! ============================================================================
//! THE CONTRACT
//! ============================================================================
//!
//! End-user face (the person building a card game / data viewer). Four verbs, total:
//!
//!     let s = Slider::new().value(0.5).at(..).elevate(..).photosynthesize(tree);  // spawn
//!     tree.write_to(s, SliderValue(0.25));                                        // update
//!     tree.subscribe(s, |t: Trigger<ValueChanged>, ..| ..);                       // listen
//!     tree.remove(s); tree.enable(s); tree.disable(s);                            // lifecycle
//!
//! A widget is ONE entity. Components in, events out. The user never sees inside:
//! no Handle, no tuple, no knowledge of children. `remove` cascades via Stem/Branch,
//! as it already does.
//!
//! Author face (foliage itself, or the end-user defining Card — the SAME tool).
//! ONE trait; Seed/Sprout/Photosynthesis collapse into it. The three-way split existed
//! only because single-bundle primitives and multi-entity composites had different
//! spawn paths — root+build erases the difference, so nothing is Photosynthesis-but-
//! not-Sprout anymore:
//!
//!     pub trait Sprout: Sized {
//!         /// the position/hierarchy/elevation seed every spawnable shares
//!         fn seed(&mut self) -> &mut LeafSprout;
//!         /// config -> components on the root entity. This IS the widget's public API.
//!         /// The library folds the seed fields in itself (GAP-3); root() returns only
//!         /// widget components.
//!         fn root(self) -> impl Bundle;
//!         /// private, STATIC skeleton. Default empty == a primitive.
//!         fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {}
//!         // provided: .at() .stem() .elevate() .with() .photosynthesize()
//!     }
//!
//! Panel/Text/Icon/Image/Line implement seed+root. Button/TextInput/Card add build.
//! `.with(..)` still folds end-user components into the same root bundle, so user data
//! coexists on the widget entity — nothing is reserved.
//!
//! PRIMITIVES OBEY THE SAME CONTRACT: each render primitive's public face is its value
//! components — IconValue on an icon entity, TextValue on a text entity, Color/Rounding
//! on a panel. Render markers stay private (Icon::new_marker et al. remain pub(crate);
//! the `Text { value }` struct-literal write path retires). Consequence: forwarding
//! config to a child is a PURE COPY, so it gets sugar — and anything that is NOT a
//! pure copy stays an explicit react, keeping the boundary visible:
//!
//!     kids.forward::<IconValue>(icon);   // react that copies root's C to a child
//!     kids.react::<TextValue>(..);       // Button's text ALSO recomputes width: not a copy
//!
//! ONE new primitive, `Children::react`:
//!
//!     kids.react::<C, _>(observer)             // C: Component + Clone; `_` = the
//!                                              // observer's inferred marker type
//!     kids.react_any::<(A, B), _>(observer)    // native bevy: observers watch tuples;
//!                                              // one body, fires when EITHER is written
//!
//! registers `observer` — a plain bevy entity-observer watching Trigger<Insert, C>, with
//! full SystemParam freedom — on the widget root, then re-inserts the root's current
//! value(s) so the observer runs once immediately. Initial state and every later
//! `write_to` share ONE code path.
//!
//! The rule that falls out (judge this hardest):
//!
//!     build  = the config-INdependent skeleton, nothing else.
//!     react  = EVERYTHING data-dependent — values AND structure.
//!
//! A dropdown's option rows, a hand's cards, a segmented control's buttons: dynamic
//! children from external data are not a feature. They are the same door a button's
//! colors go through. Reconciliation policy is author code, never library policy —
//! card_game.rs implements the same public face with a fixed pool AND with keyed
//! respawn, to prove the policy is invisible from outside.
//!
//! Events out: the `#[targeted_event]` attribute (a derive cannot inject fields, so it
//! is an attribute — same pattern as `icon_handle`). It injects the `entity` plumbing
//! field, derives bevy's EntityEvent + Clone, and generates set_target plus a
//! `new(<payload fields>)` constructor prefilled with Entity::PLACEHOLDER — the existing
//! foliage convention (Remove::new(), InsertText::new(..)). PLACEHOLDER never appears in
//! author code. (A data-type macro generating impls you'd write by hand is bevy's own
//! idiom — the no-macro line was about attribute macros rewriting observer semantics,
//! and it holds.)
//!
//!     #[targeted_event]
//!     pub struct CardPlayed { pub id: u32 }
//!     // expands to: entity field + Event/EntityEvent impls + Clone + set_target +
//!     // new(id) + an INHERENT event_target() -- so observer bodies never import the
//!     // EntityEvent trait. Emit: tree.trigger_targets(CardPlayed::new(card.id), this);
//!     // In reactions (bevy's Insert event, not ours), the target is `trigger.entity` --
//!     // bevy's own public field, also import-free.
//!
//! Events speak the CALLER's vocabulary (their ids, their indices — never child
//! entities).
//!
//! NAMING: widget markers are plain nouns (Hand, Card, Slider); their public data
//! components carry the descriptive names (Cards, CardFace, SliderValue) — the data
//! never fights the widget for a name. XSprout remains the builder-struct convention;
//! authors rarely write it, it arrives via X::new().
//!
//! Teardown: nothing to declare. Children::spawn stems everything; Remove cascades.
//! Cross-system lookup (rare — TextInput's global focus routing): the author inserts
//! their own plain component during build. Not a library concept.
//!
//! ============================================================================
//! DELETED from foliage_proper by this design
//! ============================================================================
//!   - Composite trait, composite_on_insert, handle_replace, Composite::remove
//!     (remove() is provably dead: Children::spawn stems everything, Remove cascades)
//!   - forward::<C> the free function (Insert vs Update<C> bridging — one vocabulary
//!     now; Children::forward is its principled replacement)
//!   - Primary/Secondary/Tertiary as library vocabulary (widgets own their config types)
//!   - Seed + Photosynthesis as separate traits (collapsed into Sprout)
//!   - targeted_event! + hand-written Default-with-PLACEHOLDER impls (TargetedEvent derive)
//!   - the `Text { value }` public struct-literal write path (TextValue on the entity)
//!   - tree.composite() as a load-bearing concept (one-offs: linear Children + locals,
//!     see screen.rs — music_player.rs already proved this style)
//!   - the composite require-gotcha CLASS: children spawn at photosynthesize-time, not
//!     hook-time; reactions are registered before any poke can land; nothing depends on
//!     hook ordering. (Expanded-before-Handle — the bug that bit Dropdown — becomes
//!     unrepresentable. See zoo.rs.)
//!
//! ============================================================================
//! KNOWN FRICTION, DELIBERATELY KEPT (call these out if they bother you)
//! ============================================================================
//!   - react::<C, _>(..) names C twice plus a `_` for the observer's marker type
//!     (turbofish + Trigger<Insert, C> in the closure). The macro-free price.
//!   - react (single) and react_any (tuple) are two methods: a blanket Refire impl over
//!     `C: Component` would collide with the tuple impls under coherence rules, since a
//!     foreign crate could implement Component for tuples.
//!   - config-dependent STRUCTURE cannot live in build (build is static): it routes
//!     through a reaction. RadioGroup/JoinedButtons show the consequence.
//!   - Location authoring verbosity is a separate mechanical pass (as agreed); elided
//!     as `at(/* .. */)` below wherever it would drown the signal.
//!
//! ============================================================================
//! GAPS DISCOVERED WHILE WRITING (real holes in current surface, all mechanical)
//! ============================================================================
//!   - GAP-2: CurrentInteraction (click/drag positions) must be readable by widget
//!     authors — TextInput uses it internally today; Slider needs it from outside.
//!   - GAP-3: the provided photosynthesize() must fold seed fields before root(self)
//!     consumes the sprout (mem::take on the LeafSprout — implementation detail).
//!   - GAP-4: Panel's Rounding needs per-edge support (JoinedButtons rounds only the
//!     outer edges of its first/last segments — the authoring model already expresses
//!     the position-dependence; the primitive can't draw it yet).

pub mod card_game; // the crucible: external data in, policy-free reconciliation
pub mod screen; //    one-off assembly: locals, no handles, no closure-tuples
pub mod zoo; //       button (migrated), dropdown, slider, checkbox, radio, joined
