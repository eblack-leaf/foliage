use foliage::asset::AssetKey;
use foliage::bevy_ecs;
use foliage::bevy_ecs::system::Resource;
use foliage::image_memory_handle;

#[image_memory_handle]
pub(crate) enum ImageHandles {
    Leaf,
}
#[derive(Resource)]
pub(crate) struct ImageKeys {
    pub(crate) leaf: AssetKey,
}
