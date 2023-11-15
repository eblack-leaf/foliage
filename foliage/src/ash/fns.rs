use crate::ash::render::Render;
use crate::ash::render_instructions::RenderInstructions;
use crate::ash::tag::{RenderTag, RenderTagged};
use crate::ash::{Ash};
use crate::ginkgo::Ginkgo;

pub(crate) struct PrepareFns(pub(crate) Vec<PrepareFn>);
impl PrepareFns {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
}
pub(crate) struct InstructionFns(pub(crate) Vec<InstructionFn>);

impl InstructionFns {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }
}
pub(crate) type InstructionFn = Box<fn(&mut Ash, &Ginkgo) -> Vec<RenderInstructions>>;
pub(crate) type TagFn = Box<fn() -> RenderTag>;
pub(crate) type CreateFn = Box<fn(&mut Ash, &Ginkgo)>;
pub(crate) type PrepareFn = Box<fn(&mut Ash, &Ginkgo)>;
pub(crate) struct AshLeaflet(
    pub(crate) CreateFn,
    pub(crate) PrepareFn,
    pub(crate) InstructionFn,
    pub(crate) TagFn,
);
impl AshLeaflet {
    pub(crate) fn leaf_fn<T: Render + 'static>() -> Self {
        Self(
            Box::new(Ash::register::<T>),
            Box::new(Ash::prepare::<T>),
            Box::new(Ash::instructions::<T>),
            Box::new(T::tag),
        )
    }
}


