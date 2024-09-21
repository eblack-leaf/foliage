use crate::branch::{Branch, Tree};
use crate::grid::{stem, ContextUnit, GridLocation, TokenUnit};
use crate::icon::{Icon, IconId};
use crate::interaction::{ClickInteractionListener, OnClick};
use crate::leaf::{Leaf, LeafHandle};
use crate::panel::{Panel, Rounding};
use crate::style::{Coloring, InteractiveColor};
use crate::text::{FontSize, Text, TextValue};
use crate::twig::Twig;

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
    target_handle: LeafHandle,
}
impl Button {
    pub fn new<LH: Into<LeafHandle>, ID: Into<IconId>, C: Into<Coloring>>(
        handle: LH,
        id: ID,
        c: C,
        on_click: OnClick,
    ) -> Self {
        Self {
            circle_square: ButtonShape::Square,
            coloring: c.into(),
            rounding: Default::default(),
            icon_id: id.into(),
            on_click,
            text_value: None,
            font_size: None,
            target_handle: handle.into(),
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
impl Branch for Twig<Button> {
    fn grow(self, mut tree: Tree) {
        let linked = vec![
            self.handle.extend("icon"),
            self.handle.extend("text"),
        ];
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Panel::new(self.t.rounding, self.t.coloring.background));
                l.give(
                    InteractiveColor::new(self.t.coloring.background, self.t.coloring.foreground)
                        .with_linked(linked),
                );
                let interaction_listener = match self.t.circle_square {
                    ButtonShape::Circle => ClickInteractionListener::new().as_circle(),
                    ButtonShape::Square => ClickInteractionListener::new(),
                };
                l.give(interaction_listener);
                l.give(self.t.on_click);
            })
            .named(self.handle.clone())
            .located(self.location)
            .elevation(-1),
        );
        let value = self.t.text_value.unwrap_or_default().0;
        let icon_location = if value.is_empty() {
            GridLocation::new()
                .center_x(stem().center_x())
                .center_y(stem().center_y())
                .width(stem().width() - 16.px())
                .height(stem().height() - 16.px())
        } else {
            GridLocation::new()
                .left(stem().left() + 16.px())
                .width(15.percent().width().from(stem()))
                .height(15.percent().height().from(stem()))
                .center_y(stem().center_y())
        };
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Icon::new(self.t.icon_id, self.t.coloring.foreground));
                l.stem_from(self.handle.clone());
            })
            .named("icon")
            .located(icon_location)
            .elevation(-1),
        );
        tree.add_leaf(
            Leaf::new(|l| {
                l.give(Text::new(
                    value,
                    self.t.font_size.unwrap(),
                    self.t.coloring.foreground,
                ));
                l.stem_from(self.handle.clone());
            })
            .named("text")
            .located(
                GridLocation::new()
                    .left(self.handle.extend("icon").right() + 16.px())
                    .right(stem().right() - 16.px())
                    .center_y(stem().center_y())
                    .height(90.percent().height().from(stem())),
            )
            .elevation(-1),
        )
    }
}
