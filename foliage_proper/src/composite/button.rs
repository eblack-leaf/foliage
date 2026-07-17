use crate::Component;
use crate::Trigger;
use crate::{
    anchor, Anchor, Color, Coordinates, Disengaged, EcsExtension, Elevation, Engaged, Entity,
    FocusBehavior, FontSize, Grid, GridExt, HorizontalAlignment, Icon, IconId, IconValue,
    InteractionListener, InteractionPropagation, LeafSprout, Location, Outline, Panel, Rounding,
    Sprout, Text, TextValue, Tree, VerticalAlignment, Visibility,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::system::Query;

/// A button is one entity: components in ([`ButtonStyle`], [`TextValue`], [`IconValue`],
/// [`FontSize`]), `OnClick` out. Its panel/icon/text children are private -- built by
/// `Sprout::build`, styled by reactions that run once at spawn and again on every write,
/// so there is no hand-called initial-application step. Teardown is nothing: `Remove`
/// walks every `Stem`-child recursively.
#[derive(Component, Copy, Clone)]
pub struct Button {}
impl Button {
    pub fn new() -> ButtonSprout {
        ButtonSprout::default()
    }
}

/// Button's OWN config vocabulary. The whole appearance pokes at once:
/// `tree.write_to(btn, ButtonStyle { .. })` -- icon/text/panel configure themselves.
#[derive(Component, Copy, Clone, Default)]
pub struct ButtonStyle {
    /// content color at rest (icon + text; also the fill while engaged, or always when outlined)
    pub foreground: Color,
    /// fill color at rest when un-outlined (content color while engaged)
    pub background: Color,
    pub outline: Outline,
    pub rounding: Rounding,
}

/// Interaction state as a component, so engage/disengage restyling goes through the same
/// door as config restyling -- one reaction watches `(ButtonStyle, Engagement)`.
#[derive(Component, Copy, Clone, Default)]
pub struct Engagement(pub bool);

/// Icon/text color is always foreground at rest, background when engaged, regardless of
/// outline. Panel's fill/outline is the only part that varies: filled with background when
/// there's no outline (inverting to foreground on engage); when outlined, the fill is always
/// foreground and only the outline itself toggles away on engage. One computation, shared by
/// every property/engage-state change, run by the one reaction below.
fn restyle(
    tree: &mut Tree,
    this: Entity,
    panel: Entity,
    icon: Entity,
    text: Entity,
    style: ButtonStyle,
    engaged: bool,
    icon_size: Coordinates,
) {
    let content = if engaged {
        style.background
    } else {
        style.foreground
    };
    tree.write_to(icon, content);
    tree.write_to(text, content);
    if style.outline == Outline::default() {
        let fill = if engaged {
            style.foreground
        } else {
            style.background
        };
        tree.write_to(panel, fill);
    } else {
        tree.write_to(panel, style.foreground);
        tree.write_to(
            panel,
            if engaged {
                Outline::default()
            } else {
                style.outline
            },
        );
    }
    tree.write_to(this, InteractionListener::new());
    // Full: the whole button is the icon's alignment region (see `Icon::align_render_size`),
    // centered. Anchored: the box is exactly the icon's registered footprint, its right
    // edge 8px left of the text -- exact size because the clip region derives from the box.
    let (icon_location, icon_anchor, icon_alignment) = match style.rounding {
        Rounding::Full => (
            Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ),
            Anchor::default(),
            (HorizontalAlignment::Center, VerticalAlignment::Middle),
        ),
        _ => (
            Location::new().xs(
                anchor()
                    .left()
                    .as_right()
                    .adjust(-8)
                    .with((icon_size.a() as i32).px().as_width()),
                50.pct()
                    .as_center_y()
                    .with((icon_size.b() as i32).px().as_height()),
            ),
            Anchor::new(text),
            (HorizontalAlignment::Right, VerticalAlignment::Middle),
        ),
    };
    tree.write_to(icon, icon_location);
    tree.write_to(icon, icon_anchor);
    tree.write_to(icon, icon_alignment);
    match style.rounding {
        Rounding::Full => {
            tree.write_to(panel, Rounding::Full);
            tree.write_to(text, Visibility::new(false));
        }
        _ => {
            tree.write_to(panel, Rounding::Sm);
            tree.write_to(text, Visibility::new(true));
        }
    }
}

#[derive(Default)]
pub struct ButtonSprout {
    leaf: LeafSprout,
    icon: Option<IconId>,
    text: Option<String>,
    colors: Option<(Color, Color)>,
    rounding: Option<Rounding>,
    outline: Option<i32>,
}
impl Sprout for ButtonSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        let (foreground, background) = self.colors.unwrap_or_default();
        (
            Button {},
            ButtonStyle {
                foreground,
                background,
                outline: self.outline.map(Outline::new).unwrap_or_default(),
                rounding: self.rounding.unwrap_or_default(),
            },
            TextValue(self.text.unwrap_or_default()),
            IconValue(self.icon.unwrap_or_default()),
            FontSize::default(),
            Engagement(false),
            InteractionListener::new(),
            Grid::new(1.col().gap(4), 1.row().gap(4)),
        )
    }
    fn build<T: EcsExtension>(this: Entity, tree: &mut T) {
        // static skeleton -- no config touches these spawns; every config-dependent value
        // (icon id/location, text content/width, colors, rounding) lands via the reactions'
        // first fire, in the same command batch.
        let panel = tree.branch(
            this,
            Panel::new()
                .elevate(Elevation::up(1))
                .at(Location::new().xs(
                    1.col().as_left().with(1.col().as_right()),
                    1.row().as_top().with(1.row().as_bottom()),
                ))
                .with((
                    InteractionPropagation::pass_through(),
                    FocusBehavior::ignore(),
                )),
        );
        let icon = tree.branch(
            this,
            Icon::new(0).elevate(Elevation::up(2)).with((
                InteractionPropagation::pass_through(),
                FocusBehavior::ignore(),
            )),
        );
        let text = tree.branch(
            this,
            Text::new("").elevate(Elevation::up(2)).with((
                HorizontalAlignment::Left,
                VerticalAlignment::Middle,
                InteractionPropagation::pass_through(),
                FocusBehavior::ignore(),
            )),
        );

        // one restyle for every appearance input -- engage-state and icon identity included
        // (a swapped icon can carry a different registered footprint)
        tree.react_any::<(ButtonStyle, Engagement, IconValue), _>(
            this,
            move |trigger: Trigger<Insert, (ButtonStyle, Engagement, IconValue)>,
                  styles: Query<&ButtonStyle>,
                  engagement: Query<&Engagement>,
                  icon_values: Query<&IconValue>,
                  render_sizes: bevy_ecs::system::Res<crate::IconRenderSizes>,
                  mut tree: Tree| {
                let e = trigger.event_target();
                restyle(
                    &mut tree,
                    e,
                    panel,
                    icon,
                    text,
                    *styles.get(e).unwrap(),
                    engagement.get(e).unwrap().0,
                    render_sizes.get(icon_values.get(e).unwrap().0),
                );
            },
        );
        // event -> state bridges: interaction events funnel into the same reaction door
        tree.subscribe(this, |trigger: Trigger<Engaged>, mut tree: Tree| {
            tree.entity(trigger.event_target()).insert(Engagement(true));
        });
        tree.subscribe(this, |trigger: Trigger<Disengaged>, mut tree: Tree| {
            tree.entity(trigger.event_target())
                .insert(Engagement(false));
        });

        // NOT a pure copy -- the text's width Location depends on the value -- so this
        // stays an explicit react rather than a forward.
        tree.react::<TextValue, _>(
            this,
            move |trigger: Trigger<Insert, TextValue>,
                  values: Query<&TextValue>,
                  mut tree: Tree| {
                let value = values.get(trigger.event_target()).unwrap().0.clone();
                let width = value.len();
                tree.entity(text).insert((
                    TextValue(value),
                    Location::new().xs(
                        50.pct()
                            .as_center_x()
                            .adjust(20)
                            .with(width.letters().as_width()),
                        1.row().as_top().with(1.row().as_bottom()),
                    ),
                ));
            },
        );
        // pure copies
        tree.forward::<IconValue>(this, icon);
        tree.forward::<FontSize>(this, text);
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
    pub fn colors(mut self, foreground: Color, background: Color) -> Self {
        self.colors = Some((foreground, background));
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
