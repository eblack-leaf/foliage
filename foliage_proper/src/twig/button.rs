use crate::grid::aspect::stem;
use crate::grid::location::GridLocation;
use crate::grid::unit::TokenUnit;
use crate::icon::{Icon, IconId};
use crate::interaction::{ClickInteractionListener, OnClick};
use crate::panel::{Panel, Rounding};
use crate::style::{Coloring, InteractiveColor};
use crate::text::{FontSize, Text, TextValue};
use crate::tree::{EcsExtension, Tree};
use crate::twig::{Branch, Twig};
use bevy_ecs::entity::Entity;

#[derive(Copy, Clone)]
pub(crate) enum ButtonShape {
    Circle,
    Square,
}
#[derive(Clone)]
pub struct ButtonArgs {
    circle_square: ButtonShape,
    coloring: Coloring,
    rounding: Rounding,
    icon_id: IconId,
    on_click: OnClick,
    text_value: Option<TextValue>,
    font_size: Option<FontSize>,
    outline: u32,
}
impl ButtonArgs {
    pub fn with_text<T: Into<TextValue>, FS: Into<FontSize>>(mut self, t: T, fs: FS) -> Self {
        self.text_value.replace(t.into());
        self.font_size.replace(fs.into());
        self
    }
    pub fn circle(mut self) -> Self {
        self.rounding = Rounding::all(1.0);
        self.circle_square = ButtonShape::Circle;
        self
    }
    pub fn square(mut self) -> Self {
        self.circle_square = ButtonShape::Square;
        self
    }
    pub fn rounded<R: Into<Rounding>>(mut self, rounding: R) -> Self {
        self.rounding = rounding.into();
        self
    }
    pub fn outline(mut self, outline: u32) -> Self {
        self.outline = outline;
        self
    }
}
pub struct Button {
    pub panel: Entity,
    pub icon: Entity,
    pub text: Entity,
}
impl Button {
    pub fn args<ID: Into<IconId>, C: Into<Coloring>>(
        id: ID,
        c: C,
        on_click: OnClick,
    ) -> ButtonArgs {
        ButtonArgs {
            circle_square: ButtonShape::Square,
            coloring: c.into(),
            rounding: Default::default(),
            icon_id: id.into(),
            on_click,
            text_value: None,
            font_size: None,
            outline: 0,
        }
    }
}
impl Branch for ButtonArgs {
    type Handle = Button;

    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let panel = tree.add_leaf(|l| {});
        let icon = tree.add_leaf(|l| {});
        let text = tree.add_leaf(|l| {});
        let linked = vec![icon, text];
        let interaction_listener = match twig.t.circle_square {
            ButtonShape::Circle => ClickInteractionListener::new().as_circle(),
            ButtonShape::Square => ClickInteractionListener::new(),
        };
        tree.update_leaf(panel, |l| {
            l.stem_from(twig.stem);
            l.location(twig.location);
            l.elevation(twig.elevation);
            l.give(Panel::new(twig.t.rounding, twig.t.coloring.background).outline(twig.t.outline));
            l.give(
                InteractiveColor::new(twig.t.coloring.background, twig.t.coloring.foreground)
                    .with_linked(linked),
            );
            l.give(interaction_listener);
            l.give(twig.t.on_click);
        });
        let value = twig.t.text_value.unwrap_or_default().0;
        let icon_location = if value.is_empty() {
            GridLocation::new()
                .center_x(stem().center_x())
                .center_y(stem().center_y())
                .width(24.px())
                .height(24.px())
        } else {
            GridLocation::new()
                .left(stem().left() + 16.px())
                .width(24.px())
                .height(24.px())
                .center_y(stem().center_y())
        };
        tree.update_leaf(icon, |l| {
            l.stem_from(Some(panel));
            l.location(icon_location);
            l.elevation(-1);
            l.give(Icon::new(twig.t.icon_id, twig.t.coloring.foreground));
        });
        tree.update_leaf(text, |l| {
            l.stem_from(Some(panel));
            l.elevation(-1);
            l.location(
                GridLocation::new()
                    .left(stem().left() + 48.px())
                    .right(stem().right() - 16.px())
                    .center_y(stem().center_y())
                    .height(90.percent().height().of(stem())),
            );
            if twig.t.font_size.is_some() {
                l.give(Text::new(
                    value,
                    twig.t.font_size.unwrap(),
                    twig.t.coloring.foreground,
                ));
            }
        });
        Button { panel, icon, text }
    }
}
