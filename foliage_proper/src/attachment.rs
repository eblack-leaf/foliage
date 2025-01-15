use crate::foliage::Foliage;

pub trait Attachment {
    fn attach(foliage: &mut Foliage);
}
