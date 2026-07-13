/// A safe, conservative ceiling on a single texture dimension -- WebGL2 (wgpu's wasm
/// backend) downlevel limits typically cap `max_texture_dimension_2d` well below what
/// native Vulkan/Metal/DX12 allow. Warn rather than silently emit something that only
/// fails to allocate once someone builds for wasm.
pub const SAFE_TEXTURE_DIMENSION: u32 = 2048;

pub struct MipLevel {
    pub px: u32,
    /// Coverage (alpha-only) bytes, `px * px` long, row-major.
    pub alpha: Vec<u8>,
}

/// Rasterizes `svg_bytes` into a wgpu-valid mip chain: `mips` levels, largest first,
/// each exactly half the previous (`size << level`) -- the shape wgpu's
/// `create_texture_with_data` requires, not an arbitrary bucket list. `size` is the
/// smallest (mip-count - 1) level, matching the logical on-screen render size.
pub fn rasterize_svg(svg_bytes: &[u8], size: u32, mips: u32) -> Result<Vec<MipLevel>, String> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_bytes, &opt).map_err(|e| e.to_string())?;
    let native = tree.size();

    let mut levels = Vec::with_capacity(mips as usize);
    for level in 0..mips {
        let px = size << (mips - 1 - level);
        if px > SAFE_TEXTURE_DIMENSION {
            eprintln!(
                "warning: mip level {px}px exceeds the safe wasm/WebGL2 texture ceiling \
                 ({SAFE_TEXTURE_DIMENSION}px) -- this icon set will fail to allocate on wasm"
            );
        }
        let sx = px as f32 / native.width();
        let sy = px as f32 / native.height();
        let mut pixmap =
            tiny_skia::Pixmap::new(px, px).ok_or_else(|| format!("invalid pixmap size {px}"))?;
        let transform = tiny_skia::Transform::from_scale(sx, sy);
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        // Alpha-only: the renderer tints coverage with its own Color component at draw
        // time, so the SVG's own fill/stroke color is discarded here, same as the
        // existing PNG->coverage step this replaces.
        let alpha: Vec<u8> = pixmap.pixels().iter().map(|p| p.alpha()).collect();
        levels.push(MipLevel { px, alpha });
    }
    Ok(levels)
}

/// Concatenates mip levels largest-first into the flat byte layout the engine expects.
pub fn concat_levels(levels: &[MipLevel]) -> Vec<u8> {
    levels
        .iter()
        .flat_map(|l| l.alpha.iter().copied())
        .collect()
}
