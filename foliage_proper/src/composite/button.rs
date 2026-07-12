use crate::{
    anchor, Anchor, Color, EcsExtension, Elevation, Engaged, Disengaged, FocusBehavior, FontSize,
    Grid, GridExt, HorizontalAlignment, Icon, IconId, IconValue, InteractionListener,
    InteractionPropagation, Leaf, Location, Outline, Panel, Photosynthesis, Primary, Rounding,
    Secondary, Seed, Sprout, Text, TextValue, Tree, VerticalAlignment, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;
use crate::Trigger;
use crate::Component;

/// A button is a `Panel` + `Icon` + `Text` arranged and colored together, defined entirely by
/// `ButtonSprout::photosynthesize` -- no `Handle` component, no `Composite` impl, no `forward::<C>`
/// registrations. Each child owns its own reactive closures, registered right where the child
/// is spawned and closing over that child's `Entity` directly, instead of a generic system
/// looking the child up via a shared `Query<&Handle>`. Teardown needs nothing special either:
/// `Remove` already walks every `Stem`-child recursively, so removing the button entity removes
/// panel/icon/text for free.
#[derive(Component, Copy, Clone)]
#[require(Rounding, FontSize, IconValue, Outline, Primary, Secondary)]
pub struct Button {}
impl Button {
    pub fn new() -> ButtonSprout {
        ButtonSprout::default()
    }
    pub(crate) fn new_marker() -> Self {
        Self {}
    }
}
/// Icon/text color is always primary at rest, secondary when engaged, regardless of outline.
/// Panel's fill/outline is the only part that varies: filled with secondary when there's no
/// outline (inverting to primary on engage); when outlined, the fill is always primary and only
/// the outline itself toggles away on engage (revealing the already-primary fill underneath).
/// One computation, shared by every property/engage-state change that can affect it, instead of
/// five separate functions each re-deriving a slice of the same logic.
fn recompute_colors<T: EcsExtension>(
    tree: &mut T,
    panel: Entity,
    icon: Entity,
    text: Entity,
    primary: Color,
    secondary: Color,
    outline: Outline,
    engaged: bool,
) {
    let fg = if engaged { secondary } else { primary };
    tree.write_to(icon, fg);
    tree.write_to(text, fg);
    if outline == Outline::default() {
        let bg = if engaged { primary } else { secondary };
        tree.write_to(panel, bg);
    } else {
        tree.write_to(panel, primary);
        tree.write_to(panel, if engaged { Outline::default() } else { outline });
    }
}
fn rounding_layout(round: Rounding, icon: Entity, text: Entity) -> (Location, Anchor) {
    match round {
        Rounding::Full => (
            Location::new().xs(
                50.pct().as_center_x().with(24.px().as_width()),
                50.pct().as_center_y().with(24.px().as_height()),
            ),
            Anchor::default(),
        ),
        _ => (
            Location::new().xs(
                anchor().left().as_right().adjust(-8).with(24.px().as_width()),
                50.pct().as_center_y().with(24.px().as_height()),
            ),
            Anchor::new(text),
        ),
    }
}
fn apply_rounding<T: EcsExtension>(
    tree: &mut T,
    this: Entity,
    panel: Entity,
    icon: Entity,
    text: Entity,
    round: Rounding,
) {
    tree.write_to(this, InteractionListener::new());
    let (icon_location, icon_anchor) = rounding_layout(round, icon, text);
    tree.write_to(icon, icon_location);
    match round {
        Rounding::Full => {
            tree.write_to(panel, Rounding::Full);
            tree.write_to(text, Visibility::new(false));
            tree.write_to(icon, icon_anchor);
        }
        _ => {
            tree.write_to(panel, Rounding::Sm);
            tree.write_to(text, Visibility::new(true));
            tree.write_to(icon, icon_anchor);
        }
    }
}
#[derive(Default)]
pub struct ButtonSprout {
    leaf: crate::LeafSprout,
    icon: Option<IconId>,
    text: Option<String>,
    colors: Option<(Color, Color)>,
    rounding: Option<Rounding>,
    outline: Option<i32>,
}
impl Seed for ButtonSprout {
    fn seed(&mut self) -> &mut crate::LeafSprout {
        &mut self.leaf
    }
}
impl Photosynthesis for ButtonSprout {
    // Builds children + registers reactive closures, not just one bundle -- `Button` has no
    // `bundle()`/`Sprout` membership since it's more than one entity.
    fn photosynthesize<T: EcsExtension>(self, tree: &mut T) -> Entity {
        let (primary, secondary) = self.colors.unwrap_or_default();
        let rounding = self.rounding.unwrap_or_default();
        let outline = self.outline.map(Outline::new).unwrap_or_default();
        let icon_value = self.icon.unwrap_or_default();
        let text_value = self.text.unwrap_or_default();

        let this = tree.leaf((
            Button::new_marker(),
            self.leaf.location,
            self.leaf.stem,
            self.leaf
                .elevation
                .expect("elevation not set -- call .elevate(...) before spawning"),
            IconValue(icon_value),
            TextValue(text_value.clone()),
            Primary(primary),
            Secondary(secondary),
            rounding,
            outline,
        ));
        tree.write_to(this, Grid::new(1.col().gap(4), 1.row().gap(4)));

        let panel = Panel::new()
            .elevate(Elevation::up(1))
            .at(Location::new().xs(
                1.col().as_left().with(1.col().as_right()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .stem(this)
            .with((InteractionPropagation::pass_through(), FocusBehavior::ignore()))
            .photosynthesize(tree);

        // no Location: icon's position is content-dependent (Rounding) and set below once we
        // know the real value -- an empty Location here would fail resolution immediately and
        // auto-hide it before the real one lands.
        let icon = Leaf::sprout()
            .elevate(Elevation::up(2))
            .stem(this)
            .with((Icon::new_marker(icon_value), InteractionPropagation::pass_through(), FocusBehavior::ignore()))
            .photosynthesize(tree);

        let text = Text::new(text_value.as_str())
            .elevate(Elevation::up(2))
            .at(Location::new().xs(
                50.pct().as_center_x().adjust(20).with(text_value.len().letters().as_width()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .stem(this)
            .with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                InteractionPropagation::pass_through(),
                FocusBehavior::ignore(),
            ))
            .photosynthesize(tree);

        // initial state, computed directly from the values already known above -- no fire-once
        // trigger dance needed, the values are right here.
        apply_rounding(tree, this, panel, icon, text, rounding);
        recompute_colors(tree, panel, icon, text, primary, secondary, outline, false);

        // ongoing reactivity: one closure per input, each closing over the specific children it
        // affects instead of a generic system looking them up via a shared Handle.
        tree.subscribe(this, move |trigger: Trigger<Insert, TextValue>, mut tree: Tree, values: Query<&TextValue>| {
            let value = &values.get(trigger.event_target()).unwrap().0;
            tree.entity(text)
                .insert(Text::new_marker(value.as_str()))
                .insert(Location::new().xs(
                    50.pct().as_center_x().adjust(20).with(value.len().letters().as_width()),
                    1.row().as_top().with(1.row().as_bottom()),
                ));
        });
        tree.subscribe(this, move |trigger: Trigger<Insert, FontSize>, mut tree: Tree, values: Query<&FontSize>| {
            tree.entity(text).insert(*values.get(trigger.event_target()).unwrap());
        });
        tree.subscribe(this, move |trigger: Trigger<Insert, IconValue>, mut tree: Tree, values: Query<&IconValue>| {
            tree.entity(icon).insert(Icon::new_marker(values.get(trigger.event_target()).unwrap().0));
        });
        tree.subscribe(this, move |trigger: Trigger<Insert, Rounding>, mut tree: Tree, values: Query<&Rounding>| {
            apply_rounding(&mut tree, trigger.event_target(), panel, icon, text, *values.get(trigger.event_target()).unwrap());
        });
        tree.subscribe(this, move |trigger: Trigger<Insert, Primary>, mut tree: Tree, primaries: Query<&Primary>, secondaries: Query<&Secondary>, outlines: Query<&Outline>| {
            let e = trigger.event_target();
            recompute_colors(&mut tree, panel, icon, text, primaries.get(e).unwrap().0, secondaries.get(e).unwrap().0, *outlines.get(e).unwrap(), false);
        });
        tree.subscribe(this, move |trigger: Trigger<Insert, Secondary>, mut tree: Tree, primaries: Query<&Primary>, secondaries: Query<&Secondary>, outlines: Query<&Outline>| {
            let e = trigger.event_target();
            recompute_colors(&mut tree, panel, icon, text, primaries.get(e).unwrap().0, secondaries.get(e).unwrap().0, *outlines.get(e).unwrap(), false);
        });
        tree.subscribe(this, move |trigger: Trigger<Insert, Outline>, mut tree: Tree, primaries: Query<&Primary>, secondaries: Query<&Secondary>, outlines: Query<&Outline>| {
            let e = trigger.event_target();
            recompute_colors(&mut tree, panel, icon, text, primaries.get(e).unwrap().0, secondaries.get(e).unwrap().0, *outlines.get(e).unwrap(), false);
        });
        tree.subscribe(this, move |trigger: Trigger<Engaged>, mut tree: Tree, primaries: Query<&Primary>, secondaries: Query<&Secondary>, outlines: Query<&Outline>| {
            let e = trigger.event_target();
            recompute_colors(&mut tree, panel, icon, text, primaries.get(e).unwrap().0, secondaries.get(e).unwrap().0, *outlines.get(e).unwrap(), true);
        });
        tree.subscribe(this, move |trigger: Trigger<Disengaged>, mut tree: Tree, primaries: Query<&Primary>, secondaries: Query<&Secondary>, outlines: Query<&Outline>| {
            let e = trigger.event_target();
            recompute_colors(&mut tree, panel, icon, text, primaries.get(e).unwrap().0, secondaries.get(e).unwrap().0, *outlines.get(e).unwrap(), false);
        });

        this
    }
}
impl ButtonSprout {
    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
    pub fn colors(mut self, primary: Color, secondary: Color) -> Self {
        self.colors = Some((primary, secondary));
        self
    }
    pub fn rounding(mut self, r: Rounding) -> Self {
        self.rounding = Some(r);
        self
    }
    pub fn outline(mut self, w: i32) -> Self {
        self.outline = Some(w);
        self
    }
}
