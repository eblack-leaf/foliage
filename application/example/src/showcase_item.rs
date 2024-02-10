use std::marker::PhantomData;
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::Elm;
use foliage::elm::leaf::{Leaf, Tag};
use foliage::{SceneBinding, set_descriptor};
use foliage::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage::bevy_ecs::system::{Res, SystemParamItem};
use foliage::compositor::segment::SegmentUnitDesc;
use foliage::coordinate::area::Area;
use foliage::coordinate::{CoordinateUnit, InterfaceContext};
use foliage::scene::{Anchor, Scene, SceneBinder};
use foliage::scene::align::{SceneAligner, SceneAlignment};
use foliage::text::font::MonospacedFont;
use foliage::window::ScaleFactor;

pub struct ShowcaseItem<T> {
    item: T,
}
#[derive(Bundle)]
pub struct ShowcaseItemComponents<T> {
    tag: Tag<ShowcaseItem<T>>,
}
#[derive(SceneBinding)]
pub enum ShowcaseItemBindings {
    Item,
    Desc
}
set_descriptor!(pub enum SetDescriptor { Area });
impl<T> Leaf for ShowcaseItem<T> {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(resize.in_set(Self::SetDescriptor::Area));
    }
}
fn resize() {}
impl<T> Scene for ShowcaseItem<T> {
    type Bindings = ShowcaseItemBindings;
    type Components = ();
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);

    fn bind_nodes(cmd: &mut Commands, anchor: Anchor, args: Self, external_args: &SystemParamItem<Self::ExternalArgs>, mut binder: SceneBinder<'_>) -> Self::Components {
        let metrics = metrics(anchor.0.section.area);
        binder.bind_scene(ShowcaseItemBindings::Item.into(), SceneAlignment::from((metrics.0.near(), 0.center(), 0)), metrics.2, args.item, args.item_args);
        ()
    }
}
fn metrics(area: Area<InterfaceContext>) -> (CoordinateUnit, CoordinateUnit, Area<InterfaceContext>) {

}