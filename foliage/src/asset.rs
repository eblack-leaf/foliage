use std::collections::HashMap;
pub type AssetKey = i32;
pub struct AssetContainer {
    pub assets: HashMap<AssetKey, Option<Vec<u8>>>,
}
impl AssetContainer {
    pub fn store(&mut self, id: AssetKey, bytes: Option<Vec<u8>>) {
        self.assets.insert(id, bytes);
    }
}
