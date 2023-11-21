/// Key to associate with an index for consistent referencing
#[derive(Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Debug)]
pub struct Key {
    identifier: u32,
}

impl Key {
    pub(crate) fn new(identifier: u32) -> Self {
        Self { identifier }
    }
    #[allow(unused)]
    pub fn identifier(&self) -> u32 {
        self.identifier
    }
}
/// generator for keys that are unique within a factory
pub struct KeyFactory {
    current: u32,
}

impl KeyFactory {
    pub fn new() -> Self {
        Self { current: 0 }
    }
    pub fn generate(&mut self) -> Key {
        let key = Key::new(self.current);
        self.current = self.current.checked_add(1).unwrap_or_default();
        key
    }
}
impl Default for KeyFactory {
    fn default() -> Self {
        KeyFactory::new()
    }
}
