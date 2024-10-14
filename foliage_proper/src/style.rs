use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, IntoSystemConfigs, Trigger};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{ParamSet, Query};

use crate::color::Color;
use crate::elm::{Elm, InternalStage};
use crate::interaction::{ClickInteractionListener, OnClick};
use crate::opacity::EvaluateOpacity;
use crate::tree::Tree;
use crate::Root;

pub(crate) struct Style;
impl Root for Style {
    fn attach(elm: &mut Elm) {
        elm.scheduler.main.add_systems((
            alternate_color_on_engage.in_set(InternalStage::Clean),
            alternate_color_on_engage.in_set(InternalStage::SecondClean),
        ));
    }
}
#[derive(Component, Clone)]
pub struct InteractiveColor {
    pub base: Color,
    pub alternate_color: Color,
    pub linked: Vec<Entity>,
}
impl InteractiveColor {
    pub fn new<B: Into<Color>, A: Into<Color>>(b: B, a: A) -> Self {
        Self {
            base: b.into(),
            alternate_color: a.into(),
            linked: vec![],
        }
    }
    pub fn with_linked(mut self, linked: Vec<Entity>) -> Self {
        self.linked = linked;
        self
    }
}
pub(crate) fn alternate_triggered(trigger: Trigger<OnClick>) {
    todo!()
}
pub(crate) fn alternate_color_on_engage(
    mut alts: ParamSet<(
        Query<
            (
                Entity,
                &mut Color,
                &InteractiveColor,
                &ClickInteractionListener,
            ),
            Or<(
                Changed<ClickInteractionListener>,
                Changed<Color>,
                Changed<InteractiveColor>,
            )>,
        >,
        Query<&mut Color>,
    )>,
    mut tree: Tree,
) {
    let mut set = Vec::new();
    for (entity, mut color, alt, listener) in alts.p0().iter_mut() {
        if listener.engaged_start() && !listener.engaged_end() {
            for linked in alt.linked.iter() {
                set.push((*linked, alt.base));
            }
            *color = alt.alternate_color;
            tree.entity(entity).insert(EvaluateOpacity {});
        } else if listener.engaged_end() {
            for linked in alt.linked.iter() {
                set.push((*linked, alt.alternate_color));
            }
            *color = alt.base;
            tree.entity(entity).insert(EvaluateOpacity {});
        }
    }
    for (e, c) in set {
        if let Ok(mut color) = alts.p1().get_mut(e) {
            *color = c;
            tree.entity(e).insert(EvaluateOpacity {});
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
