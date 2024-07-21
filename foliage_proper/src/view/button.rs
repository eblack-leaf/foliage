use crate::grid::{Grid, GridCoordinate, GridPlacement};
use crate::panel::{Panel, Rounding};
use crate::style::Coloring;
use crate::view::{View, Viewable};
#[derive(Copy, Clone)]
pub(crate) enum ButtonShape {
    Circle,
    Square,
}
pub struct Button {
    circle_square: ButtonShape,
    coloring: Coloring,
}
impl Button {
    pub fn new() -> Self {
        Self {
            circle_square: ButtonShape::Square,
            coloring: Default::default(),
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
    pub fn colored<C: Into<Coloring>>(mut self, c: C) -> Self {
        self.coloring = c.into();
        self
    }
}
impl Viewable for Button {
    fn build(self, view: &mut View) {
        view.config_grid(Grid::new(3, 1));
        view.bind(
            "panel",
            GridPlacement::new(0.percent().to(100.percent()), 0.percent().to(100.percent())),
            None,
            |b| {
                let rounding = match self.circle_square {
                    ButtonShape::Circle => Rounding::all(1.0),
                    ButtonShape::Square => Rounding::default(),
                };
                b.give_attr(Panel::new(rounding, self.coloring.background));
            },
        );
        // text + icon (no conditional + IconButton + TextButton)
    }
}
