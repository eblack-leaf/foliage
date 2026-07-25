use crate::Trigger;
use crate::{
    Animation, Color, Component, Ease, EcsExtension, Elevation, Entity, FocusBehavior, Grid,
    GridExt, Icon, IconId, InteractionListener, InteractionPropagation, LeafSprout, Location,
    OnClick, Outline, Panel, Rounding, Sequence, Sprout, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A toggle is one entity: components in ([`ToggleState`], [`ToggleStyle`],
/// [`ToggleSprout::check_icon`]), [`Toggled`] out. Clicks and programmatic `ToggleState`
/// writes go through the same door -- Button's `Engagement` pattern with persistent instead
/// of momentary state. Track and knob follow Material 3: an outlined track with a small
/// knob at rest, a filled track with a large checked knob once on -- the same fill-vs-
/// outline exclusivity `Checkbox`'s box uses for its own on/off switch.
#[derive(Component, Copy, Clone)]
pub struct Toggle {}
impl Toggle {
    pub fn new() -> ToggleSprout {
        ToggleSprout::default()
    }
}

/// Toggle's public value channel: write it to flip the switch programmatically.
#[derive(Component, Copy, Clone, Default)]
pub struct ToggleState(pub bool);

/// Toggle's OWN config vocabulary, poked as one unit:
/// `tree.write_to(toggle, ToggleStyle { .. })`.
#[derive(Component, Copy, Clone, Default)]
pub struct ToggleStyle {
    /// track fill while on
    pub on_fill: Color,
    /// track border while off (track has no fill of its own then -- whatever's behind it
    /// shows through, same as `Checkbox`'s unchecked box)
    pub off_outline: Color,
    /// knob color while on
    pub knob_on: Color,
    /// knob color while off
    pub knob_off: Color,
}

/// The checkmark glyph -- structural (which icon, if any), not a color/rounding style
/// knob, so it gets its own component, same split `Checkbox` uses between
/// `CheckboxConfig`/`CheckboxStyle`. Set once at spawn; `build`'s reaction on this is what
/// actually spawns the icon child. `None` means no icon was configured -- no child gets
/// spawned at all, not a defunct hidden one.
#[derive(Component, Copy, Clone)]
pub(crate) struct ToggleConfig {
    check_icon: Option<IconId>,
}

/// Private child registry: the style reaction needs the check icon's entity id, which only
/// exists after the config reaction below has run.
#[derive(Component, Copy, Clone)]
pub(crate) struct ToggleHandle {
    check: Option<Entity>,
}

/// Emitted at the toggle root whenever [`ToggleState`] changes (click or programmatic
/// write) -- including once at spawn with the initial value, since the reaction that fires
/// it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Toggled {
    pub on: bool,
}

/// The knob is near-full-track-height while on (a half-track-width pill inset 4px from the
/// edges), shrinking to roughly half that diameter while off -- the same size drop
/// Material's switch uses to keep the empty state reading as "smaller than the track it
/// sits in" rather than just "the same knob, slid left." Pure percentage anchors (no fixed
/// pixel insets) on both axes so the shrink scales with the track instead of being crushed
/// at the small track heights this app actually uses -- a fixed-pixel inset that reads fine
/// on the 30px test track over-shrinks a real ~22px row's knob to a speck.
fn knob_location(on: bool) -> Location {
    if on {
        Location::new().xs(
            50.pct()
                .as_left()
                .adjust(2)
                .with(100.pct().as_right().adjust(-4)),
            0.pct()
                .as_top()
                .adjust(4)
                .with(100.pct().as_bottom().adjust(-4)),
        )
    } else {
        Location::new().xs(
            12.5.pct().as_left().with(37.5.pct().as_right()),
            0.pct()
                .as_top()
                .adjust(6)
                .with(100.pct().as_bottom().adjust(-6)),
        )
    }
}

pub struct ToggleSprout {
    leaf: LeafSprout,
    on: bool,
    check_icon: Option<IconId>,
    style: ToggleStyle,
}
impl Default for ToggleSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            on: false,
            check_icon: None,
            style: ToggleStyle::default(),
        }
    }
}
impl ToggleSprout {
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }
    /// The checkmark glyph shown on the knob while on -- optional; the library ships no
    /// icons of its own, and the track's own fill-vs-outline switch already carries the
    /// on/off state without it (see `Toggle`'s own docs).
    pub fn check_icon<ID: Into<IconId>>(mut self, id: ID) -> Self {
        self.check_icon = Some(id.into());
        self
    }
    pub fn colors(
        mut self,
        on_fill: Color,
        off_outline: Color,
        knob_on: Color,
        knob_off: Color,
    ) -> Self {
        self.style = ToggleStyle {
            on_fill,
            off_outline,
            knob_on,
            knob_off,
        };
        self
    }
}
impl Sprout for ToggleSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Toggle {},
            ToggleState(self.on),
            ToggleConfig {
                check_icon: self.check_icon,
            },
            self.style,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton -- fill-vs-outline switches per state, but the track panel itself
        // (and its rounding) never depends on config.
        let track = tree.branch(
            this,
            Panel::new()
                .rounding(Rounding::Full)
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
        // no Location: state-dependent, set by that reaction's first fire
        let knob = tree.branch(
            this,
            Panel::new()
                .rounding(Rounding::Full)
                .elevate(Elevation::up(2))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                    // the check icon below is parented to the knob (not the toggle root) so
                    // its position rides the knob's own slide/resize -- that requires the
                    // knob to carry a Grid of its own, the same way any composite gives a
                    // non-root child Grid::default() before branching further children off it.
                    Grid::default(),
                )),
        );

        // structure: the checkmark glyph is author config, so it's spawned inside the
        // reaction on ToggleConfig's insert rather than the static skeleton above --
        // registered before the style reaction below, so its ToggleHandle write lands
        // before that reaction's own first fire (same ordering Checkbox uses between its
        // CheckboxConfig and CheckboxStyle reactions). Parented to the knob itself so its
        // Location rides along with the knob's own slide/resize for free.
        tree.react::<ToggleConfig, _>(
            this,
            move |trigger: Trigger<Insert, ToggleConfig>,
                  configs: Query<&ToggleConfig>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let check_icon = configs.get(e).unwrap().check_icon;
                let check = check_icon.map(|icon| {
                    tree.branch(
                        knob,
                        Icon::new(icon)
                            .at(Location::new().xs(
                                20.pct().as_left().with(80.pct().as_right()),
                                20.pct().as_top().with(80.pct().as_bottom()),
                            ))
                            .elevate(Elevation::up(3))
                            .with((
                                InteractionPropagation::pass_through(),
                                FocusBehavior::ignore(),
                            )),
                    )
                });
                tree.write_to(e, ToggleHandle { check });
            },
        );

        // input: click -> state flip. ONE component write; drawing happens in the reaction.
        tree.on_click(
            this,
            |trigger: Trigger<OnClick>, states: Query<&ToggleState>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.write_to(e, ToggleState(!states.get(e).unwrap().0));
            },
        );

        // render: state/style -> track fill-vs-outline + knob color/placement + checkmark
        // visibility. The first fire places the knob directly (there's no prior Location to
        // slide from); every later state change slides/resizes it -- captured-flag state,
        // the ProjectCard-image pattern. Track exclusivity mirrors Checkbox: a positive
        // `Outline` means stroke-only (whatever's behind the track shows through, matching
        // Material's "off" track), `Outline::default()` (-1) means solid fill.
        let mut place_directly = true;
        tree.react_any::<(ToggleState, ToggleStyle), _>(
            this,
            move |trigger: Trigger<Insert, (ToggleState, ToggleStyle)>,
                  states: Query<&ToggleState>,
                  styles: Query<&ToggleStyle>,
                  handles: Query<&ToggleHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let on = states.get(e).unwrap().0;
                let style = *styles.get(e).unwrap();
                if on {
                    tree.write_to(track, (style.on_fill, Outline::default()));
                    tree.write_to(knob, style.knob_on);
                } else {
                    tree.write_to(track, (style.off_outline, Outline::new(2)));
                    tree.write_to(knob, style.knob_off);
                }
                if let Some(check) = handles.get(e).unwrap().check {
                    tree.write_to(check, (style.on_fill, Visibility::new(on)));
                }
                if place_directly {
                    tree.write_to(knob, knob_location(on));
                    place_directly = false;
                } else {
                    Sequence::new(&mut tree).animate(
                        Animation::new(knob_location(on))
                            .targeting(knob)
                            .start(0)
                            .finish(150)
                            .eased(Ease::EMPHASIS),
                    );
                }
            },
        );
        // state -> event bridge; kept out of the restyle reaction so ToggleStyle writes
        // don't announce a state change that didn't happen.
        tree.react::<ToggleState, _>(
            this,
            |trigger: Trigger<Insert, ToggleState>, states: Query<&ToggleState>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.trigger_targets(Toggled::new(states.get(e).unwrap().0), e);
            },
        );
    }
}
