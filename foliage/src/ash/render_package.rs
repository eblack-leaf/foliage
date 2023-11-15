use crate::ash::render::Render;
use crate::ginkgo::Ginkgo;
use std::collections::HashMap;
use crate::ash::render_instructions::{RenderInstructions, RenderInstructionsRecorder};

pub struct RenderPackage<T> {
    pub data: T,
    pub(crate) instructions: Option<RenderInstructions>,
    dirty: bool,
}

impl<T> RenderPackage<T> {
    pub(crate) fn new(resources: T) -> Self {
        Self {
            data: resources,
            instructions: None,
            dirty: true,
        }
    }
    pub fn flag_dirty(&mut self) {
        self.dirty = true;
    }
}

pub struct RenderPackageManager<Renderer: Render> {
    pub renderer: Renderer,
    pub packages: HashMap<Renderer::Key, RenderPackage<Renderer::RenderPackageResources>>,
}

impl<Renderer: Render> RenderPackageManager<Renderer> {
    pub(crate) fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            packages: HashMap::new(),
        }
    }
    pub fn new_package(
        &mut self,
        key: Renderer::Key,
        package_resources: Renderer::RenderPackageResources,
    ) {
        self.packages
            .insert(key, RenderPackage::new(package_resources));
    }
    pub(crate) fn instructions(&mut self, ginkgo: &Ginkgo) -> Vec<RenderInstructions> {
        let mut instructions = vec![];
        for (_, package) in self.packages.iter_mut() {
            if package.dirty {
                package.instructions.replace(self.renderer.record_package(
                    package,
                    RenderInstructionsRecorder::new(ginkgo),
                    ginkgo.viewport.as_ref().unwrap(),
                ));
                package.dirty = false;
            }
            instructions.push(package.instructions.clone().unwrap());
        }
        instructions
    }
}
