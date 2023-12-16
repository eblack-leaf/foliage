use std::collections::HashSet;
use bevy_ecs::prelude::Entity;
use crate::coordinate::InterfaceContext;
use crate::coordinate::section::Section;

pub struct Segment {
    pub section: Section<InterfaceContext>,
    pub entities: HashSet<Entity>,
}

pub struct SegmentHandle(pub i32);
//
pub struct Workflow(pub i32);

pub struct WorkflowTransition(pub HashSet<Entity>);