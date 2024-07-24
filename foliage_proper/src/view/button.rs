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
    text_value: TextValue,
    font_size: FontSize,
}
impl Button {
    pub fn new<ID: Into<IconId>, T: Into<TextValue>, FS: Into<FontSize>, C: Into<Coloring>>(
        id: ID,
        t: T,
        fs: FS,
        c: C,
        on_click: OnClick,
    ) -> Self {
        Self {
            circle_square: ButtonShape::Square,
            coloring: c.into(),
            rounding: Default::default(),
            icon_id: id.into(),
            on_click,
            text_value: t.into(),
            font_size: fs.into(),
        }
    }
    pub fn circle(mut self) -> Self {
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
            view.target_handle.extend("text").into(),
        ];
        view.bind(
            "panel",
            GridPlacement::new(0.percent().to(100.percent()), 0.percent().to(100.percent())),
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
        view.bind(
            "icon",
            GridPlacement::new(16.px().to(48.px()), 1.row().to(1.row())).offset_layer(-1),
            None,
            |b| {
                b.give_attr(Icon::new(self.icon_id, self.coloring.foreground));
            },
        );
        view.bind(
            "text",
            GridPlacement::new(56.px().to(3.col()), 1.row().to(1.row())).offset_layer(-1),
            None,
            |b| {
                b.give_attr(Text::new(
                    self.text_value.0,
                    self.font_size,
                    self.coloring.foreground,
                ))
            },
        )
    }
}
