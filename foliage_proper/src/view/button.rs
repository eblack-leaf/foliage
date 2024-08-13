use crate::grid::{Grid, GridCoordinate, GridPlacement};
use crate::icon::{Icon, IconId};
use crate::interaction::{ClickInteractionListener, OnClick};
use crate::panel::{Panel, Rounding};
use crate::style::{Coloring, InteractiveColor};
use crate::text::{FontSize, Text, TextValue};
use crate::view::{View, Viewable};

#[derive(Copy, Clone)]
pub(crate) enum ButtonShape {
    Circle,
    Square,
}
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
impl Viewable for Button {
    fn build(self, view: &mut View) {
        view.config_grid(Grid::new(3, 1));
        let linked = vec![
            view.target_handle.extend("icon"),
            view.target_handle.extend("text"),
        ];
        view.bind(
            "panel",
            GridPlacement::new(0.percent().to(100.percent()), 0.percent().to(100.percent())),
            -1,
            None,
            |b| {
                b.give_attr(Panel::new(self.rounding, self.coloring.background));
                b.give_attr(
                    InteractiveColor::new(self.coloring.background, self.coloring.foreground)
                        .with_linked(linked),
                );
                let interaction_listener = match self.circle_square {
                    ButtonShape::Circle => ClickInteractionListener::new().as_circle(),
                    ButtonShape::Square => ClickInteractionListener::new(),
                };
                b.give_attr(interaction_listener);
                b.give_attr(self.on_click);
            },
        );
        let icon_horizontal = if self.text_value.is_some() {
            10.percent().span(24.px())
        } else {
            1.col().to(3.col())
        };
        view.bind(
            "icon",
            GridPlacement::new(icon_horizontal, 1.row().to(1.row())),
            -1,
            None,
            |b| {
                b.give_attr(Icon::new(self.icon_id, self.coloring.foreground));
            },
        );
        let text_horizontal = if self.text_value.is_some() {
            25.percent().to(85.percent())
        } else {
            0.percent().to(0.percent())
        };
        view.bind(
            "text",
            GridPlacement::new(text_horizontal, 1.row().to(1.row())),
            -1,
            None,
            |b| {
                if let Some(t) = self.text_value {
                    b.give_attr(Text::new(
                        t.0,
                        self.font_size.unwrap(),
                        self.coloring.foreground,
                    ));
                }
            },
        )
    }
}
