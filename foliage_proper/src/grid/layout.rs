use bevy_ecs::system::Resource;

#[derive(Resource, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Layout {
    Sm,
    Md,
    Lg,
    Xl,
}
