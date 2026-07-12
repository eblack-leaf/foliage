//! THE CRUCIBLE. A minimal card game: draw, play (click a card), sort.
//!
//! Read `end_user` first — that is the ENTIRE surface a game developer touches.
//! Then the author space: Card (a widget), Hand (a widget made of widgets, pooled
//! policy), KeyedHand (same public face, respawn-by-key policy).

use foliage::*;
use std::collections::HashMap;

// ===========================================================================
// END-USER SPACE
// ===========================================================================

/// The game's OWN truth. Not a foliage concept. Lives wherever the user wants —
/// a Resource here; could be a server snapshot, a save file, anything.
#[derive(Resource, Default)]
struct Game {
    deck: Vec<CardData>,
    hand: Vec<CardData>,
}

#[derive(Clone, PartialEq)]
struct CardData {
    id: u32, // stable identity, minted by the game, opaque to foliage
    rank: u8,
    suit: Suit,
}

fn end_user(tree: &mut Tree) {
    // spawn: same chain as a Panel
    let hand = Hand::new()
        .at(Location::new().xs(
            1.col().as_left().with(12.col().as_right()),
            14.row().as_top().with(16.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .photosynthesize(tree);

    let draw = Button::new()
        .text("Draw")
        .at(/* .. */)
        .elevate(Elevation::up(1))
        .photosynthesize(tree);

    // hand data -> forget. The ONLY way state ever reaches the widget.
    tree.on_click(
        draw,
        move |_: Trigger<OnClick>, mut game: ResMut<Game>, mut tree: Tree| {
            if let Some(card) = game.deck.pop() {
                game.hand.push(card);
            }
            tree.write_to(hand, Cards::from_game(&game.hand));
        },
    );

    // listen: the widget answers in GAME vocabulary — the id the game minted,
    // never a child entity, never a slot index.
    tree.subscribe(
        hand,
        move |t: Trigger<CardPlayed>, mut game: ResMut<Game>, mut tree: Tree| {
            let id = t.event().id;
            game.hand.retain(|c| c.id != id);
            // ... resolve game effects against `game` ...
            tree.write_to(hand, Cards::from_game(&game.hand));
        },
    );

    // reorder is not an API: it is the same write with the Vec in a different order.
    // (a Sort button just sorts game.hand and does the identical write_to.)

    // multi-view: the same truth, two renderings, is two writes to two widgets —
    // e.g. tree.write_to(pile_view, Discards::from(&game.discard)) alongside the
    // hand. No coordination concept needed; they are independent entities.
}

// ===========================================================================
// AUTHOR SPACE — Card (a leaf widget; nested inside Hand below)
// ===========================================================================

/// Card's public data component. Poke it, the card redraws.
/// (Widget markers are plain nouns; data components get the descriptive names.)
#[derive(Component, Clone, Default)]
pub struct CardFace {
    pub rank: u8,
    pub suit: Suit,
}

#[derive(Component)]
pub struct Card {}
impl Card {
    pub fn new() -> CardSprout {
        CardSprout::default()
    }
}

#[derive(Default)]
pub struct CardSprout {
    leaf: LeafSprout,
    face: CardFace,
}
impl CardSprout {
    /// builder config folds into root() — one bundle, no spawn-then-patch
    pub fn face(mut self, rank: u8, suit: Suit) -> Self {
        self.face = CardFace { rank, suit };
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
            self.face,
            InteractionListener::new(),
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        // static skeleton — no config touches this
        let panel = kids.spawn(
            Panel::new()
                .rounding(Rounding::Md)
                .at(/* fills root */)
                .elevate(Elevation::up(1))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );
        let rank = kids.spawn(
            Text::new("")
                .size(FontSize::new(20))
                .at(/* top-left corner */)
                .elevate(Elevation::up(2))
                .with(InteractionPropagation::pass_through()),
        );
        let pip = kids.spawn(
            Icon::new(0)
                .at(/* centered */)
                .elevate(Elevation::up(2))
                .with(InteractionPropagation::pass_through()),
        );

        // ALL state application: runs once now (fire-once) and on every write_to.
        // Primitives obey the same contract — TextValue/IconValue/Color written
        // straight onto them; no markers, no special forms.
        kids.react::<CardFace, _>(
            move |t: Trigger<Insert, CardFace>, faces: Query<&CardFace>, mut tree: Tree| {
                let face = faces.get(t.entity).unwrap();
                tree.write_to(rank, TextValue(face.rank_label()));
                tree.write_to(rank, face.suit.color());
                tree.write_to(pip, IconValue(face.suit.icon()));
                tree.write_to(panel, Color::gray(100));
            },
        );
    }
}

// ===========================================================================
// AUTHOR SPACE — Hand (pooled policy)
// ===========================================================================

/// Hand's public data component. THIS AUTHOR chose its shape; the library
/// reserves nothing, so the end-user's own components coexist on the same entity.
#[derive(Component, Clone, Default)]
pub struct Cards(Vec<HandCard>);
#[derive(Clone)]
pub struct HandCard {
    pub id: u32,
    pub rank: u8,
    pub suit: Suit,
}
impl Cards {
    pub fn from_game(cards: &[CardData]) -> Self {
        /* map CardData -> HandCard */
        Cards::default()
    }
}

/// Event out. Caller vocabulary: the id, nothing internal. The derive generates
/// set_target + `CardPlayed::new(id)` (PLACEHOLDER prefilled, never written by hand).
#[targeted_event]
#[derive(Copy)]
pub struct CardPlayed {
    pub id: u32,
}

#[derive(Component)]
pub struct Hand {}
const MAX_HAND: usize = 10;

impl Hand {
    pub fn new() -> HandSprout {
        HandSprout::default()
    }
}
#[derive(Default)]
pub struct HandSprout {
    leaf: LeafSprout,
}

impl Sprout for HandSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Hand {},
            Cards::default(),
            Grid::new(MAX_HAND.col().gap(8), 1.row()),
        )
    }
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        // POLICY (this author's): a fixed pool of card slots. Entities never die;
        // empty slots are disabled. Nothing in the library knows or cares.
        let slots: Vec<Entity> = (0..MAX_HAND)
            .map(|i| {
                kids.spawn(
                    Card::new()
                        .at(Location::new().xs(
                            (i + 1).col().as_left().with((i + 1).col().as_right()),
                            1.row().as_top().with(1.row().as_bottom()),
                        ))
                        .elevate(Elevation::up(1)),
                )
            })
            .collect();

        // per-slot interaction, wired once at build: slot click -> CardPlayed at root,
        // translated into caller vocabulary (the id currently shown in that slot)
        for (i, slot) in slots.iter().copied().enumerate() {
            kids.tree().on_click(
                slot,
                move |_: Trigger<OnClick>, cards: Query<&Cards>, mut tree: Tree| {
                    if let Some(card) = cards.get(this).unwrap().0.get(i) {
                        tree.trigger_targets(CardPlayed::new(card.id), this);
                    }
                },
            );
        }

        // ALL state application. Fires once now with Cards::default() (all slots off),
        // then on every write_to(hand, Cards(..)) forever.
        kids.react::<Cards, _>(
            move |t: Trigger<Insert, Cards>, cards: Query<&Cards>, mut tree: Tree| {
                let cards = cards.get(t.entity).unwrap();
                for (i, slot) in slots.iter().copied().enumerate() {
                    match cards.0.get(i) {
                        Some(c) => {
                            // poking the nested widget's PUBLIC component — a widget
                            // author uses other widgets exactly as an end-user does
                            tree.write_to(slot, CardFace { rank: c.rank, suit: c.suit });
                            tree.enable(slot);
                        }
                        None => tree.disable(slot),
                    }
                }
            },
        );
    }
}

// ===========================================================================
// SAME PUBLIC FACE, DIFFERENT POLICY — KeyedHand
// ===========================================================================
// Stable entity per card id: survivors keep their entity across writes, so a
// Location animation can carry a card to its new slot on reorder — THE reason
// an author picks this policy. Entities leave when their card does.
// End-user code above works against this widget UNCHANGED.

#[derive(Component)]
pub struct KeyedHand {}
#[derive(Default)]
pub struct KeyedHandSprout {
    leaf: LeafSprout,
}

impl Sprout for KeyedHandSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            KeyedHand {},
            Cards::default(),
            Grid::new(MAX_HAND.col().gap(8), 1.row()),
        )
    }
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        // no static skeleton at all: structure IS the data.
        // reconciliation state is plain FnMut capture — not a library feature.
        let mut live: HashMap<u32, Entity> = HashMap::new();
        kids.react::<Cards, _>(
            move |t: Trigger<Insert, Cards>, cards: Query<&Cards>, mut tree: Tree| {
                let this = t.entity;
                let cards = cards.get(this).unwrap();
                // leavers: Remove cascades through the Card's insides for free
                live.retain(|id, e| {
                    let keep = cards.0.iter().any(|c| c.id == *id);
                    if !keep {
                        tree.remove(*e);
                    }
                    keep
                });
                for (i, c) in cards.0.iter().enumerate() {
                    let at = Location::new().xs(
                        (i + 1).col().as_left().with((i + 1).col().as_right()),
                        1.row().as_top().with(1.row().as_bottom()),
                    );
                    match live.get(&c.id) {
                        // survivor: animate to its new slot (stable entity = animatable)
                        Some(e) => {
                            tree.animate(Animation::new(at).start(0).finish(250).targeting(*e));
                        }
                        // arrival: nested widget, config through its builder
                        None => {
                            let e = Children::new(this, &mut tree).spawn(
                                Card::new()
                                    .face(c.rank, c.suit)
                                    .at(at)
                                    .elevate(Elevation::up(1)),
                            );
                            live.insert(c.id, e);
                        }
                    }
                }
            },
        );
        // (per-card click wiring: same on_click pattern as the pooled version,
        //  registered at spawn inside the arrival arm — elided)
    }
}

// --- scaffolding the spec references but doesn't care about ---
#[derive(Copy, Clone, Default, PartialEq)]
pub enum Suit {
    #[default]
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}
impl Suit {
    fn icon(self) -> IconId {
        /* .. */
        0
    }
    fn color(self) -> Color {
        /* .. */
        Color::gray(100)
    }
}
impl CardFace {
    fn rank_label(&self) -> String {
        /* "A", "2".."10", "J", "Q", "K" */
        String::new()
    }
}
