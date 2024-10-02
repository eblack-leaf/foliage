use crate::grid::{stem, GridLocation, TokenUnit};
use crate::icon::{Icon, IconId};
use crate::interaction::{ClickInteractionListener, OnClick};
use crate::leaf::Leaf;
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
pub struct Button {
    circle_square: ButtonShape,
    coloring: Coloring,
    rounding: Rounding,
    icon_id: IconId,
    on_click: OnClick,
    text_value: Option<TextValue>,
    font_size: Option<FontSize>,
}
impl Button {
    pub fn new<ID: Into<IconId>, C: Into<Coloring>>(id: ID, c: C, on_click: OnClick) -> Self {
        Self {
            circle_square: ButtonShape::Square,
            coloring: c.into(),
            rounding: Default::default(),
            icon_id: id.into(),
            on_click,
            text_value: None,
            font_size: None,
        }
    }
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
}
pub struct ButtonHandle {
    pub panel: Entity,
    pub icon: Entity,
    pub text: Entity,
}
impl Branch for Button {
    type Handle = ButtonHandle;

    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let panel = tree.add_leaf(Leaf::new().stem_from(twig.stem).elevation(twig.elevation));
        let icon = tree.add_leaf(Leaf::new().elevation(-1).stem_from(Some(panel)));
        let text = tree.add_leaf(Leaf::new().stem_from(Some(panel)).elevation(-1));
        let linked = vec![icon, text];
        let interaction_listener = match twig.t.circle_square {
            ButtonShape::Circle => ClickInteractionListener::new().as_circle(),
            ButtonShape::Square => ClickInteractionListener::new(),
        };
        tree.entity(panel)
            .insert(Panel::new(twig.t.rounding, twig.t.coloring.background))
            .insert(twig.location)
            .insert(
                InteractiveColor::new(twig.t.coloring.background, twig.t.coloring.foreground)
                    .with_linked(linked),
            )
            .insert(interaction_listener)
            .insert(twig.t.on_click);
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
        tree.entity(icon)
            .insert(Icon::new(twig.t.icon_id, twig.t.coloring.foreground))
            .insert(icon_location);
        tree.entity(text).insert(
            GridLocation::new()
                .left(stem().left() + 48.px())
                .right(stem().right() - 16.px())
                .center_y(stem().center_y())
                .height(90.percent().height().of(stem())),
        );
        if twig.t.font_size.is_some() {
            tree.entity(text).insert(Text::new(
                value,
                twig.t.font_size.unwrap(),
                twig.t.coloring.foreground,
            ));
        }
        ButtonHandle { panel, icon, text }
    }
}
