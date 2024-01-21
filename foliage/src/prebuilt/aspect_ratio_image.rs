use crate::compositor::layout::AspectRatio;
use crate::coordinate::area::Area;
use crate::coordinate::section::Section;
use crate::coordinate::{InterfaceContext, NumericalContext};
use crate::differential::Despawn;
use crate::elm::config::{ElmConfiguration, ExternalSet};
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::image::{Image, ImageId};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};

#[derive(Component, Copy, Clone)]
pub struct ImageDimensions(pub Area<NumericalContext>);
impl<A: Into<Area<NumericalContext>>> From<A> for ImageDimensions {
    fn from(value: A) -> Self {
        Self(value.into())
    }
}
#[derive(Bundle)]
pub struct AspectRatioImage {
    tag: Tag<Self>,
    dims: ImageDimensions,
    id: ImageId,
}
pub enum AspectRatioImageBindings {
    Image,
}
impl From<AspectRatioImageBindings> for SceneBinding {
    fn from(value: AspectRatioImageBindings) -> Self {
        Self(value as i32)
    }
}
set_descriptor!(
    pub enum AspectRatioImageSets {
        Area,
    }
);
fn metrics(area: Area<InterfaceContext>, dims: ImageDimensions) -> Area<InterfaceContext> {
    let ratio = AspectRatio::new(dims.0);
    let mut attempted_width = area.width;
    let mut attempted_height = area.width * ratio.reciprocal();
    while attempted_height > area.height {
        attempted_width -= 1f32;
        attempted_height = attempted_width * ratio.reciprocal();
    }
    Area::new(attempted_width, attempted_height)
}
impl Leaf for AspectRatioImage {
    type SetDescriptor = AspectRatioImageSets;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems(resize.in_set(Self::SetDescriptor::Area));
    }
}
fn resize(
    scenes: Query<
        (
            &SceneHandle,
            &Area<InterfaceContext>,
            &Despawn,
            &ImageId,
            &ImageDimensions,
        ),
        (
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<ImageId>,
                Changed<ImageDimensions>,
            )>,
            With<Tag<AspectRatioImage>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut images: Query<(&mut ImageId, &mut Area<InterfaceContext>), Without<Tag<AspectRatioImage>>>,
) {
    for (handle, area, despawn, id, dims) in scenes.iter() {
        if despawn.should_despawn() {
            continue;
        }
        coordinator.update_anchor_area(*handle, *area);
        let image = coordinator.binding_entity(
            &handle
                .access_chain()
                .target(AspectRatioImageBindings::Image),
        );
        *images.get_mut(image).unwrap().0 = *id;
        let aligned_dims = metrics(*area, *dims);
        *images.get_mut(image).unwrap().1 = aligned_dims;
    }
}
pub struct AspectRatioImageArgs {
    id: ImageId,
    dims: ImageDimensions,
    view: Option<Section<InterfaceContext>>,
}
impl AspectRatioImageArgs {
    pub fn new<ID: Into<ImageId>, DIM: Into<ImageDimensions>>(
        id: ID,
        dim: DIM,
        v: Option<Section<InterfaceContext>>,
    ) -> Self {
        Self {
            id: id.into(),
            dims: dim.into(),
            view: v,
        }
    }
}
impl Scene for AspectRatioImage {
    type Bindings = AspectRatioImageBindings;
    type Args<'a> = AspectRatioImageArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        _anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind(
            AspectRatioImageBindings::Image,
            (0.center(), 0.center(), 0),
            Image::new(args.id).with_view(args.view),
            cmd,
        );
        Self {
            tag: Tag::new(),
            dims: args.dims,
            id: args.id,
        }
    }
}
