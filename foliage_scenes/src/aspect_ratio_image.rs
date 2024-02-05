use foliage_macros::InnerSceneBinding;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::component::Component;
use foliage_proper::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, Or, With, Without};
use foliage_proper::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_proper::compositor::layout::AspectRatio;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::{InterfaceContext, NumericalContext};
use foliage_proper::differential::Despawn;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::image::{Image, ImageId};
use foliage_proper::scene::align::SceneAligner;
use foliage_proper::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage_proper::set_descriptor;
#[derive(Component, Copy, Clone)]
pub struct ImageDimensions(pub Area<NumericalContext>);
impl<A: Into<Area<NumericalContext>>> From<A> for ImageDimensions {
    fn from(value: A) -> Self {
        Self(value.into())
    }
}
#[derive(Bundle)]
pub struct AspectRatioImageComponents {
    tag: Tag<Self>,
    dims: ImageDimensions,
    id: ImageId,
}
#[derive(InnerSceneBinding)]
pub enum AspectRatioImageBindings {
    Image,
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
            With<Tag<AspectRatioImageComponents>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut images: Query<
        (&mut ImageId, &mut Area<InterfaceContext>),
        Without<Tag<AspectRatioImageComponents>>,
    >,
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
#[derive(Clone)]
pub struct AspectRatioImage {
    id: ImageId,
    dims: ImageDimensions,
}
impl AspectRatioImage {
    pub fn new<ID: Into<ImageId>, DIM: Into<ImageDimensions>>(id: ID, dim: DIM) -> Self {
        Self {
            id: id.into(),
            dims: dim.into(),
        }
    }
}
impl Scene for AspectRatioImage {
    type Bindings = AspectRatioImageBindings;
    type Components = AspectRatioImageComponents;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        _anchor: Anchor,
        args: Self,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self::Components {
        binder.bind(
            AspectRatioImageBindings::Image,
            (0.center(), 0.center(), 0),
            Image::new(args.id),
            cmd,
        );
        Self::Components {
            tag: Tag::new(),
            dims: args.dims,
            id: args.id,
        }
    }
}