use crate::Foliage;

pub trait Attachment {
    fn attach(foliage: &mut Foliage);
}
