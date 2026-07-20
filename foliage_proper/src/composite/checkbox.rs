use crate::Trigger;
use crate::{
    Color, Component, EcsExtension, Elevation, Entity, FocusBehavior, Grid, GridExt, Icon, IconId,
    InteractionListener, InteractionPropagation, LeafSprout, Location, OnClick, Outline, Panel,
    Rounding, Sprout, Tree, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A checkbox is one entity: components in ([`CheckboxState`], [`CheckboxStyle`],
/// [`CheckboxSprout::check_icon`]), [`Checked`] out. Same `Engagement`-free
/// persistent-state shape as `Toggle` (click flips one component, a reaction draws the
/// result) -- the visual difference is the whole point: an empty outlined box at rest, a
/// filled box + checkmark icon once checked. The checkmark is an author-supplied icon --
/// the library ships no icons of its own, so `.check_icon(..)` is required.
#[derive(Component, Copy, Clone)]
pub struct Checkbox {}
impl Checkbox {
    pub fn new() -> CheckboxSprout {
        CheckboxSprout::default()
    }
}

/// Checkbox's public value channel: write it to check/uncheck programmatically.
#[derive(Component, Copy, Clone, Default)]
pub struct CheckboxState(pub bool);

/// Checkbox's OWN config vocabulary, poked as one unit.
#[derive(Component, Copy, Clone, Default)]
pub struct CheckboxStyle {
    /// box border color while unchecked
    pub outline: Color,
    /// box fill color while checked
    pub fill: Color,
    /// checkmark icon's color
    pub check: Color,
    pub rounding: Rounding,
}

/// The checkmark glyph -- structural (which icon), not a color/rounding style knob, so it
/// gets its own component, same split as `Modal`'s `ModalConfig`/`ModalStyle`. Set once at
/// spawn, `build`'s reaction on this is what actually spawns the icon child (`build` itself
/// has no way to read the sprout's own config back -- this component is that door).
#[derive(Component, Copy, Clone)]
pub(crate) struct CheckboxConfig {
    check_icon: IconId,
}

/// Private child registry: the style reaction needs the check icon's entity id, which only
/// exists after the config reaction below has run.
#[derive(Component, Copy, Clone)]
pub(crate) struct CheckboxHandle {
    check: Entity,
}

/// Emitted at the checkbox root whenever [`CheckboxState`] changes (click or programmatic
/// write) -- including once at spawn with the initial value, since the reaction that fires
/// it re-fires initial state like every `react`.
#[foliage_macros::targeted_event]
#[derive(Copy)]
pub struct Checked {
    pub on: bool,
}

pub struct CheckboxSprout {
    leaf: LeafSprout,
    on: bool,
    check_icon: Option<IconId>,
    style: CheckboxStyle,
}
impl Default for CheckboxSprout {
    fn default() -> Self {
        Self {
            leaf: LeafSprout::default(),
            on: false,
            check_icon: None,
            style: CheckboxStyle::default(),
        }
    }
}
impl CheckboxSprout {
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }
    /// The checkmark glyph -- required; the library ships no icons of its own.
    pub fn check_icon<ID: Into<IconId>>(mut self, id: ID) -> Self {
        self.check_icon = Some(id.into());
        self
    }
    pub fn colors(mut self, outline: Color, fill: Color, check: Color) -> Self {
        self.style.outline = outline;
        self.style.fill = fill;
        self.style.check = check;
        self
    }
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.style.rounding = r;
        self
    }
}
impl Sprout for CheckboxSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Checkbox {},
            CheckboxState(self.on),
            CheckboxConfig {
                check_icon: self
                    .check_icon
                    .expect("Checkbox::check_icon(..) is required"),
            },
            self.style,
            Grid::default(),
            InteractionListener::new(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton -- the box itself never depends on config.
        let panel = tree.branch(
            this,
            Panel::new()
                .outline(2)
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

        // structure: the checkmark glyph is author config, so it's spawned inside the
        // reaction on CheckboxConfig's insert rather than the static skeleton above --
        // registered before the style reaction below, so its CheckboxHandle write lands
        // before that reaction's own first fire (same ordering Modal uses between its
        // ModalConfig and ModalStyle reactions).
        tree.react::<CheckboxConfig, _>(
            this,
            move |trigger: Trigger<Insert, CheckboxConfig>,
                  configs: Query<&CheckboxConfig>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let check_icon = configs.get(e).unwrap().check_icon;
                let check = tree.branch(
                    e,
                    Icon::new(check_icon)
                        .at(Location::new().xs(
                            20.pct().as_left().with(80.pct().as_right()),
                            20.pct().as_top().with(80.pct().as_bottom()),
                        ))
                        .elevate(Elevation::up(2))
                        .with((
                            InteractionPropagation::pass_through(),
                            FocusBehavior::ignore(),
                        )),
                );
                tree.write_to(e, CheckboxHandle { check });
            },
        );

        tree.on_click(
            this,
            |trigger: Trigger<OnClick>, states: Query<&CheckboxState>, mut tree: Tree| {
                let e = trigger.event_target();
                tree.write_to(e, CheckboxState(!states.get(e).unwrap().0));
            },
        );

        // render: state/style -> box fill-vs-outline + checkmark visibility. Panel's own
        // fill/outline exclusivity does the state switch for free: outline-only (no fill)
        // while unchecked, solid fill (no outline) while checked -- see `panel/mod.rs`, a
        // positive `Outline` value means stroke-only, `Outline::default()` (-1) means solid.
        tree.react_any::<(CheckboxState, CheckboxStyle), _>(
            this,
            move |trigger: Trigger<Insert, (CheckboxState, CheckboxStyle)>,
                  states: Query<&CheckboxState>,
                  styles: Query<&CheckboxStyle>,
                  handles: Query<&CheckboxHandle>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                let on = states.get(e).unwrap().0;
                let style = *styles.get(e).unwrap();
                if on {
                    tree.write_to(panel, (style.fill, Outline::default(), style.rounding));
                } else {
                    tree.write_to(panel, (style.outline, Outline::new(2), style.rounding));
                }
                let check = handles.get(e).unwrap().check;
                tree.write_to(check, (style.check, Visibility::new(on)));
            },
        );
        // state -> event bridge; kept out of the render reaction so style-only writes don't
        // announce a state change that didn't happen.
        tree.react::<CheckboxState, _>(
            this,
            |trigger: Trigger<Insert, CheckboxState>,
             states: Query<&CheckboxState>,
             mut tree: Tree| {
                let e = trigger.event_target();
                tree.trigger_targets(Checked::new(states.get(e).unwrap().0), e);
            },
        );
    }
}
