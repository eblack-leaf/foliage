use bevy_ecs::entity::Entity;
use bevy_ecs::system::Commands;
use std::collections::HashMap;

pub struct Scene<Args> {
    args: Option<Args>,
    nodes: HashMap<SceneBinding, Box<SceneNode<Args>>>,
}
pub type SceneNode<Args> = fn(&Args, &mut Commands) -> Entity;
pub type SceneBinding = u32;
impl<Args> Scene<Args> {
    pub fn new() -> Self {
        Self {
            args: None,
            nodes: HashMap::new(),
        }
    }
    pub fn with_node(mut self, binding: SceneBinding, scene_node: SceneNode<Args>) -> Self {
        self.nodes.insert(binding, Box::new(scene_node));
        self
    }
    pub fn set_args(&mut self, args: Args) {
        self.args.replace(args);
    }
    pub fn spawn_with(&self, cmd: &mut Commands) -> Vec<(SceneBinding, Entity)> {
        let mut entities = vec![];
        for (binding, node) in self.nodes.iter() {
            let entity = node(self.args.as_ref().unwrap(), cmd);
            entities.push((*binding, entity));
        }
        entities
    }
}
