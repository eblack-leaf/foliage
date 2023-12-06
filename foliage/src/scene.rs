use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::differential::Despawn;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Commands, Query};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Copy, Clone)]
pub struct SceneVisibility(pub bool); // TODO incorporate into visibility check
impl Default for SceneVisibility {
    fn default() -> Self {
        SceneVisibility(true)
    }
}
#[derive(Bundle)]
pub struct SceneAlignment {
    alignment: AlignmentCoordinate,
    anchor: AlignmentAnchor,
    binding: SceneBinding,
    layout_binding: SceneLayoutBinding,
    visibility: SceneVisibility,
}
impl SceneAlignment {
    pub fn new(
        ac: AlignmentCoordinate,
        anchor: AlignmentAnchor,
        binding: SceneBinding,
        layout_binding: SceneLayoutBinding,
    ) -> Self {
        Self {
            alignment: ac,
            anchor,
            binding,
            layout_binding,
            visibility: SceneVisibility::default(),
        }
    }
}
#[derive(Bundle, Copy, Clone)]
pub struct AlignmentCoordinate {
    pub ha: HorizontalAlignment,
    pub va: VerticalAlignment,
    pub la: LayerAlignment,
}
#[derive(
    Component, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Default,
)]
pub struct SceneBinding(pub u32);
#[derive(
    Component, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Default,
)]
pub struct SceneLayoutBinding(pub u32);
impl From<u32> for SceneBinding {
    fn from(value: u32) -> Self {
        SceneBinding(value)
    }
}
impl From<u32> for SceneLayoutBinding {
    fn from(value: u32) -> Self {
        SceneLayoutBinding(value)
    }
}
#[derive(Component, Default)]
pub struct SceneNodes(pub HashMap<SceneBinding, Entity>);
impl SceneNodes {
    pub fn release(&mut self, cmd: &mut Commands) {
        self.0.drain().for_each(|n| {
            cmd.entity(n.1).insert(Despawn::new(true));
        });
    }
}
#[derive(Component, Default)]
pub struct SceneLayout(pub HashMap<SceneLayoutBinding, AlignmentCoordinate>);
impl SceneLayout {
    pub fn get(&self, binding: SceneLayoutBinding) -> AlignmentCoordinate {
        *self.0.get(&binding).unwrap()
    }
}
#[derive(Bundle)]
pub struct Scene {
    pub anchor: AlignmentAnchor,
    pub entities: SceneNodes,
    pub layout: SceneLayout,
    pub visibility: SceneVisibility,
    pub despawn: Despawn,
}
impl Scene {
    pub fn new(anchor: Coordinate<InterfaceContext>, layout: SceneLayout) -> Self {
        Self {
            anchor: AlignmentAnchor(anchor),
            entities: SceneNodes::default(),
            layout,
            visibility: SceneVisibility::default(),
            despawn: Despawn::default(),
        }
    }
}
#[derive(Component)]
pub struct SceneBindRequest<T: Bundle>(pub Vec<(SceneBinding, SceneLayoutBinding, T)>);
impl<T: Bundle> SceneBindRequest<T> {
    pub fn new<SB: Into<SceneBinding>, SLB: Into<SceneLayoutBinding>>(
        mut bundles: Vec<(SB, SLB, T)>,
    ) -> Self {
        Self(
            bundles
                .drain(..)
                .map(|(sb, slb, t)| (sb.into(), slb.into(), t))
                .collect(),
        )
    }
}
pub(crate) fn bind<T: Bundle + 'static>(
    mut requests: Query<(
        Entity,
        &AlignmentAnchor,
        &mut SceneBindRequest<T>,
        &mut SceneNodes,
        &SceneLayout,
    )>,
    mut cmd: Commands,
) {
    for (entity, anchor, mut bind_request, mut nodes, layout) in requests.iter_mut() {
        for (scene_binding, layout_binding, bundle) in bind_request.0.drain(..) {
            // TODO batch?
            let requested_entity = cmd
                .spawn(bundle.chain(SceneAlignment::new(
                    layout.get(layout_binding),
                    *anchor,
                    scene_binding,
                    layout_binding,
                )))
                .id();
            nodes.0.insert(scene_binding, requested_entity);
        }
        cmd.entity(entity).remove::<SceneBindRequest<T>>();
    }
}
pub(crate) fn place(
    mut aligned: Query<
        (
            &AlignmentAnchor,
            &HorizontalAlignment,
            &VerticalAlignment,
            &mut Position<InterfaceContext>,
            &Area<InterfaceContext>,
        ),
        Or<(
            Changed<AlignmentAnchor>,
            Changed<HorizontalAlignment>,
            Changed<VerticalAlignment>,
            Changed<Position<InterfaceContext>>,
            Changed<Area<InterfaceContext>>,
        )>,
    >,
) {
    for (anchor, ha, va, mut pos, area) in aligned.iter_mut() {
        let x = ha.calc(anchor.section(), *area);
        let y = va.calc(anchor.section(), *area);
        *pos = (x, y).into();
    }
}
pub(crate) fn place_layer(
    mut aligned: Query<
        (&AlignmentAnchor, &LayerAlignment, &mut Layer),
        Or<(
            Changed<AlignmentAnchor>,
            Changed<LayerAlignment>,
            Changed<Layer>,
        )>,
    >,
) {
    for (anchor, la, mut layer) in aligned.iter_mut() {
        *layer = la.calc(anchor.layer());
    }
}
#[derive(Bundle)]
pub struct ChainedBundle<T: Bundle, S: Bundle> {
    pub original: T,
    pub extension: S,
}

impl<T: Bundle, S: Bundle> ChainedBundle<T, S> {
    pub fn new(t: T, s: S) -> Self {
        Self {
            original: t,
            extension: s,
        }
    }
}

pub trait BundleChain
where
    Self: Bundle + Sized,
{
    fn chain<B: Bundle>(self, b: B) -> ChainedBundle<Self, B>;
}

impl<I: Bundle> BundleChain for I {
    fn chain<B: Bundle>(self, b: B) -> ChainedBundle<I, B> {
        ChainedBundle::new(self, b)
    }
}
#[derive(Copy, Clone, Component)]
pub struct AlignmentAnchor(pub Coordinate<InterfaceContext>);
impl AlignmentAnchor {
    pub fn section(&self) -> Section<InterfaceContext> {
        self.0.section
    }
    pub fn layer(&self) -> Layer {
        self.0.layer
    }
}
#[derive(Component, Copy, Clone)]
pub enum HorizontalAlignment {
    Center(CoordinateUnit),
    Left(CoordinateUnit),
    Right(CoordinateUnit),
}
impl HorizontalAlignment {
    pub fn calc(
        &self,
        scene_section: Section<InterfaceContext>,
        target: Area<InterfaceContext>,
    ) -> CoordinateUnit {
        match self {
            HorizontalAlignment::Center(alignment) => {
                scene_section.center().x - target.width / 2f32 + alignment
            }
            HorizontalAlignment::Left(alignment) => scene_section.left() + alignment,
            HorizontalAlignment::Right(alignment) => scene_section.right() - alignment,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub enum VerticalAlignment {
    Center(CoordinateUnit),
    Top(CoordinateUnit),
    Bottom(CoordinateUnit),
}
impl VerticalAlignment {
    pub fn calc(
        &self,
        scene_section: Section<InterfaceContext>,
        target: Area<InterfaceContext>,
    ) -> CoordinateUnit {
        match self {
            VerticalAlignment::Center(alignment) => {
                scene_section.center().y - target.width / 2f32 + alignment
            }
            VerticalAlignment::Top(alignment) => scene_section.top() + alignment,
            VerticalAlignment::Bottom(alignment) => scene_section.bottom() - alignment,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct LayerAlignment(pub Layer);
impl LayerAlignment {
    pub fn calc(&self, scene: Layer) -> Layer {
        self.0 + scene
    }
}