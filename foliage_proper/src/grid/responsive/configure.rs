use crate::grid::responsive::resolve::{ResolvedConfiguration, ResolvedPoints};
use crate::grid::responsive::{
    PointExceptions, ResponsiveConfigurationException, ResponsivePoints, ResponsiveSection,
};
use crate::grid::token::AspectValueWrapper;
use crate::layout::Layout;
use bevy_ecs::component::StorageType::SparseSet;
use bevy_ecs::component::{Component, ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::DeferredWorld;

#[derive(Copy, Clone, Default)]
pub struct ConfigureFromLayout {}

impl ConfigureFromLayout {
    fn on_insert(mut world: DeferredWorld, entity: Entity, _c: ComponentId) {
        let layout = world.get_resource::<Layout>().unwrap().clone();
        if let Some(base) = world.entity(entity).get::<ResponsiveSection>().cloned() {
            let mut to_use = base.configurations.clone();
            if let Some(exceptions) = world
                .entity(entity)
                .get::<ResponsiveConfigurationException>()
            {
                for (a, b) in exceptions.exceptions.iter() {
                    if a.layout.contains(layout) {
                        let mut aspect = b.clone();
                        match aspect.aspects[0].value {
                            AspectValueWrapper::Existing => {
                                let config = a.config.value();
                                aspect.aspects[0].value =
                                    if base.configurations[config].1.aspects[0].aspect
                                        == aspect.aspects[0].aspect
                                    {
                                        base.configurations[config].1.aspects[0].value.clone()
                                    } else {
                                        debug_assert_eq!(
                                            base.configurations[config].1.aspects[1].aspect,
                                            aspect.aspects[0].aspect
                                        );
                                        base.configurations[config].1.aspects[1].value.clone()
                                    };
                            }
                            _ => {}
                        }
                        match aspect.aspects[1].value {
                            AspectValueWrapper::Existing => {
                                let config = a.config.value();
                                aspect.aspects[1].value =
                                    if base.configurations[config].1.aspects[0].aspect
                                        == aspect.aspects[1].aspect
                                    {
                                        base.configurations[config].1.aspects[0].value.clone()
                                    } else {
                                        debug_assert_eq!(
                                            base.configurations[config].1.aspects[1].aspect,
                                            aspect.aspects[1].aspect
                                        );
                                        base.configurations[config].1.aspects[1].value.clone()
                                    };
                            }
                            _ => {}
                        }
                        to_use[a.config.value()].1 = aspect;
                    }
                }
            }
            if let Some(mut resolved) = world.get_mut::<ResolvedConfiguration>(entity) {
                resolved.configurations = to_use;
            }
        }
        if let Some(base) = world.entity(entity).get::<ResponsivePoints>().cloned() {
            let mut to_use = base.configurations.clone();
            if let Some(exceptions) = world.entity(entity).get::<PointExceptions>() {
                for (a, b) in exceptions.exceptions.iter() {
                    if a.layout.contains(layout) {
                        let config = a.pac.value();
                        let mut aspect = b.clone();
                        if aspect.count == 0 {
                            continue;
                        }
                        for i in 0..2 {
                            match aspect.aspects[i].value {
                                AspectValueWrapper::Existing => {
                                    aspect.aspects[i].value =
                                        if base.configurations[config].1.aspects[0].aspect
                                            == aspect.aspects[i].aspect
                                        {
                                            base.configurations[config].1.aspects[0].value.clone()
                                        } else {
                                            debug_assert_eq!(
                                                base.configurations[config].1.aspects[1].aspect,
                                                aspect.aspects[i].aspect
                                            );
                                            base.configurations[config].1.aspects[1].value.clone()
                                        }
                                }
                                _ => {}
                            }
                        }
                        to_use[config].1 = aspect;
                    }
                }
            }
            if let Some(mut resolved) = world.get_mut::<ResolvedPoints>(entity) {
                resolved.configurations = to_use;
            }
        }
    }
}

impl Component for ConfigureFromLayout {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(ConfigureFromLayout::on_insert);
    }
}
