use bevy_ecs::bundle::Bundle;

#[derive(Bundle)]
pub struct ChainedBundle<T: Bundle, S: Bundle> {
    pub original: T,
    pub extension: S,
}

impl<T: Bundle, S: Bundle> ChainedBundle<T, S> {
    pub fn new(t: T, s: S) -> Self {
        Self {
            original: t,
            extension: s,
        }
    }
}

pub trait BundleChain
where
    Self: Bundle + Sized,
{
    fn chain<B: Bundle>(self, b: B) -> ChainedBundle<Self, B>;
}

impl<I: Bundle> BundleChain for I {
    fn chain<B: Bundle>(self, b: B) -> ChainedBundle<I, B> {
        ChainedBundle::new(self, b)
    }
}