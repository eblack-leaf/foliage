use crate::foliage::Foliage;

/// A unit of engine setup: the components, systems, resources and observers one feature
/// needs, installed together.
///
/// The built-in primitives implement this and are attached by
/// [`Foliage::new`](crate::Foliage::new). An app or library implements it to install its
/// own, then calls [`Foliage::attach`](crate::Foliage::attach) once at startup.
pub trait Attachment {
    fn attach(foliage: &mut Foliage);
}
