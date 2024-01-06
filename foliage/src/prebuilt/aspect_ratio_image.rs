use crate::coordinate::area::Area;
use crate::coordinate::{InterfaceContext, NumericalContext};
use crate::differential::Despawn;
use crate::elm::leaf::Tag;
use crate::image::{Image, ImageId};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use crate::set_descriptor;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Commands;
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
    todo!()
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
}
impl Scene for AspectRatioImage {
    type Bindings = AspectRatioImageBindings;
    type Args<'a> = AspectRatioImageArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind(
            AspectRatioImageBindings::Image,
            (0.center(), 0.center(), 0),
            Image::new(args.id),
            cmd,
        );
        Self {
            tag: Tag::new(),
            dims: args.dims,
            id: args.id,
        }
    }
}