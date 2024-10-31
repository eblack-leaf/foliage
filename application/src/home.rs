use foliage::bevy_ecs;
use foliage::bevy_ecs::prelude::Resource;
use foliage::color::{Color, Grey, Monochromatic};
use foliage::grid::aspect::{screen, stem};
use foliage::grid::responsive::evaluate::{ScrollContext, Scrollable};
use foliage::grid::responsive::ResponsiveLocation;
use foliage::grid::unit::TokenUnit;
use foliage::leaf::{EvaluateCore, Leaf};
use foliage::panel::{Panel, Rounding};
use foliage::text::{FontSize, Text};
use foliage::tree::Tree;
use foliage::twig::{Branch, Twig};
#[derive(Resource)]
pub(crate) struct IdTable {}

pub(crate) struct Home {}

impl Branch for Home {
    type Handle = IdTable;

    fn grow(twig: Twig<Self>, tree: &mut Tree) -> Self::Handle {
        let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
        eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, \
        quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\
         Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu \
         fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in \
         culpa qui officia deserunt mollit anim id est laborum.";
        let scroll_view = tree
            .spawn(Leaf::new().elevation(10))
            .insert(Panel::new(Rounding::all(0.0), Grey::base()))
            .insert(Scrollable::default())
            .insert(
                ResponsiveLocation::new()
                    .top(screen().top() + 16.px())
                    .bottom(screen().bottom() - 16.px())
                    .left(screen().left() + 16.px())
                    .width(400.px()),
            )
            .insert(EvaluateCore::recursive())
            .id();
        let text = tree
            .spawn(Leaf::new().stem(Some(scroll_view)).elevation(-1))
            .insert(Text::new(long_text, FontSize::new(24), Color::WHITE))
            .insert(
                ResponsiveLocation::new()
                    .top(stem().top())
                    .auto_height()
                    .left(stem().left())
                    .width(120.percent().width().of(stem())),
            )
            .insert(ScrollContext::new(scroll_view))
            .insert(EvaluateCore::recursive())
            .id();
        let after_text = tree
            .spawn(Leaf::new().stem(Some(text)).elevation(0))
            .insert(Panel::new(Rounding::all(0.2), Color::WHITE))
            .insert(
                ResponsiveLocation::new()
                    .top(stem().bottom() + 16.px())
                    .height(50.px())
                    .left(stem().left())
                    .width(100.percent().width().of(stem())),
            )
            .insert(ScrollContext::new(scroll_view))
            .insert(EvaluateCore::recursive())
            .id();
        IdTable {}
    }
}
