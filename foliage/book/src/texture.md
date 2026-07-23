# Texture

Every raster-backed primitive ([Image](./image.md), [Icon](./icon.md), and glyph
rendering behind [Text](./text.md)) needs GPU texture memory, and none of them want to
own a whole dedicated texture per instance -- a screen with fifty icons shouldn't mean
fifty separate GPU textures and fifty draw calls. `TextureAtlas` is the shared packing
structure underneath all three.

```rust
// foliage_proper/src/texture.rs
pub(crate) struct TextureAtlas<Key: Hash + Clone, Referrer: Hash + Clone, TexelData: ...> {
    texture: Texture,
    view: TextureView,
    partitions: HashMap<Key, PartitionInfo>,
    possible_locations: HashSet<AtlasLocation>,
    block_size: Coordinates,
    references: HashMap<Key, HashSet<Referrer>>,
    entries: HashMap<Key, AtlasEntry<TexelData>>,
    ...
}
```

One real `wgpu::Texture`, subdivided into a grid of fixed-size `block_size` cells. `Key`
identifies *what* is stored (an icon id, a glyph key, an image asset key); `Referrer`
identifies *who's currently using it* (an entity), tracked separately via `references`
so an atlas slot is only actually freed once nothing refers to it anymore
(`remove_reference` calls `remove_entry` only when the reference set empties).

## Reference counting, not per-entity textures

```rust
// foliage_proper/src/texture.rs
pub(crate) fn add_reference(&mut self, key: Key, referrer: Referrer)
pub(crate) fn remove_reference(&mut self, key: Key, referrer: Referrer) {
    self.references.get_mut(&key).unwrap().remove(&referrer);
    if self.references.get(&key).unwrap().is_empty() {
        self.remove_entry(key);
    }
}
```

Two `Text` entities rendering the same glyph, or two `Icon` entities using the same
icon id, share one atlas slot -- the second reference doesn't re-upload the same texel
data, it just adds itself to that key's `references` set. This is what keeps a UI with
many repeated icons/glyphs cheap on GPU memory regardless of how many entities render
them.

## Growing without losing existing entries

```rust
// foliage_proper/src/texture.rs
pub(crate) fn resolve(&mut self, ginkgo: &Ginkgo) -> (HashSet<Key>, bool) {
    // if new entries exceed remaining capacity: allocate a bigger backing texture,
    // re-pack, and report every already-partitioned key as `changed`
}
```

When the atlas runs out of room, `resolve` allocates a new, larger backing texture and
re-derives every existing entry's texture coordinates against it -- returning the full
set of keys whose coordinates just moved, so whatever's referencing them (a `Differential`
queue entry per glyph/icon) can re-queue with the updated coordinates rather than
silently rendering from stale, now-wrong UV offsets.
