use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Commands, Query, Res};

use crate::color::Color;
use crate::elm::{Elm, ScheduleMarkers};
use crate::interaction::ClickInteractionListener;
use crate::leaf::{IdTable, LeafHandle};
use crate::Root;

pub(crate) struct Style;
impl Root for Style {
    fn define(elm: &mut Elm) {
        elm.scheduler
            .main
            .add_systems(alternate_color_on_engage.in_set(ScheduleMarkers::Preparation));
    }
}
#[derive(Component, Clone)]
pub struct InteractiveColor {
    pub base: Color,
    pub alternate_color: Color,
    pub linked: Vec<LeafHandle>,
}
impl InteractiveColor {
    pub fn new<B: Into<Color>, A: Into<Color>>(b: B, a: A) -> Self {
        Self {
            base: b.into(),
            alternate_color: a.into(),
            linked: vec![],
        }
    }
    pub fn with_linked(mut self, linked: Vec<LeafHandle>) -> Self {
        self.linked = linked;
        self
    }
}
pub(crate) fn alternate_color_on_engage(
    mut alts: Query<
        (&mut Color, &InteractiveColor, &ClickInteractionListener),
        Or<(
            Changed<ClickInteractionListener>,
            Changed<Color>,
            Changed<InteractiveColor>,
        )>,
    >,
    mut cmd: Commands,
    id_table: Res<IdTable>,
) {
    for (mut color, alt, listener) in alts.iter_mut() {
        if listener.engaged_start() && !listener.engaged_end() {
            for linked in alt.linked.iter() {
                let entity = id_table.lookup_leaf(linked.clone()).unwrap();
                cmd.entity(entity).insert(alt.base);
            }
            *color = alt.alternate_color;
        } else if listener.engaged_end() {
            for linked in alt.linked.iter() {
                let entity = id_table.lookup_leaf(linked.clone()).unwrap();
                cmd.entity(entity).insert(alt.alternate_color);
            }
            *color = alt.base;
        }
    }
}
#[derive(Copy, Clone, Component)]
pub struct Coloring {
    pub foreground: Color,
    pub background: Color,
}
impl Coloring {
    pub fn new<A: Into<Color>, B: Into<Color>>(fg: A, bg: B) -> Self {
        Self {
            foreground: fg.into(),
            background: bg.into(),
        }
    }
}
