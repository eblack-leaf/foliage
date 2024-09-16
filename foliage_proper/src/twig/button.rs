use crate::grid::{ContextUnit, Grid, GridLocation, TokenUnit};
use crate::icon::{Icon, IconId};
use crate::interaction::{ClickInteractionListener, OnClick};
use crate::leaf::Leaf;
use crate::panel::{Panel, Rounding};
use crate::style::{Coloring, InteractiveColor};
use crate::text::{FontSize, Text, TextValue};
use crate::twig::{TwigDef, TwigStem};

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
impl TwigDef for Button {
    fn grow(self, twig_stem: &mut TwigStem) {
        twig_stem.config_grid(Grid::new(3, 1));
        let linked = vec![
            twig_stem.target_handle.extend("icon"),
            twig_stem.target_handle.extend("text"),
        ];
        twig_stem.bind(
            Leaf::new(|l| {
                l.give(Panel::new(self.rounding, self.coloring.background));
                l.give(
                    InteractiveColor::new(self.coloring.background, self.coloring.foreground)
                        .with_linked(linked),
                );
                let interaction_listener = match self.circle_square {
                    ButtonShape::Circle => ClickInteractionListener::new().as_circle(),
                    ButtonShape::Square => ClickInteractionListener::new(),
                };
                l.give(interaction_listener);
                l.give(self.on_click);
            })
            .named("panel")
            .located(GridLocation::new())
            .elevation(-1),
        );

        let value = self.text_value.unwrap_or_default().0;
        let icon_location = if value.is_empty() {
            GridLocation::new()
                .center_x(twig_stem.target_handle.clone().center_x())
                .center_y(twig_stem.target_handle.clone().center_y())
                .width(twig_stem.target_handle.clone().width() - 16.px())
                .height(twig_stem.target_handle.clone().height() - 16.px())
        } else {
            GridLocation::new()
                .left(twig_stem.target_handle.clone().left() + 16.px())
                .width(15.percent_width(twig_stem.target_handle.clone()))
                .height(15.percent_width(twig_stem.target_handle.clone()))
                .center_y(twig_stem.target_handle.clone().center_y())
        };
        twig_stem.bind(
            Leaf::new(|l| l.give(Icon::new(self.icon_id, self.coloring.foreground)))
                .named("icon")
                .located(icon_location)
                .elevation(-1),
        );
        twig_stem.bind(
            Leaf::new(|l| {
                l.give(Text::new(
                    value,
                    self.font_size.unwrap(),
                    self.coloring.foreground,
                ))
            })
            .named("text")
            .located(
                GridLocation::new()
                    .left(twig_stem.target_handle.extend("icon").right() + 16.px())
                    .right(twig_stem.target_handle.clone().right() - 16.px())
                    .center_y(twig_stem.target_handle.clone().center_y())
                    .height(90.percent_height(twig_stem.target_handle.clone())),
            )
            .elevation(-1),
        )
    }
}
