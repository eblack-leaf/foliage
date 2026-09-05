//! Bakes an SVG into the field foliage draws a mark from.
//!
//! A mark has no size of its own: the same artwork is a 16px affordance and a 96px empty state.
//! So it is not rasterised at a size but stored once as a distance to its own edge, and
//! reconstructed at whatever box a layout hands it. What comes out is a square RGBA field --
//! multi-channel in RGB, so a median across the three reconstructs a corner a single channel
//! would round off, and a plain signed distance in A.
//!
//! Two things are produced together and have to stay together: the `.icon` files, and a Rust
//! module that registers them as a [`Marks`] set. The generated module reads them with
//! `include_bytes!`, so it sits in the same directory they do.
//!
//! [`Marks`]: https://eblack-leaf.github.io/foliage/api/foliage/trait.Marks.html
//!
//! # Baking one field
//!
//! ```no_run
//! # fn f() -> Result<(), String> {
//! let svg = std::fs::read("check.svg").map_err(|e| e.to_string())?;
//! let baked = foliage_icons::bake(&svg, foliage_icons::Bake::new(48, 3.0))?;
//! std::fs::write("check.icon", &baked.rgba).map_err(|e| e.to_string())?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

mod codegen;
mod msdf;
mod naming;
mod preview;

pub use codegen::generate;
pub use msdf::{Bake, Baked, bake};
pub use preview::preview;

use std::fs;
use std::path::Path;

/// Bakes every `.svg` in a directory and writes the set that registers them.
///
/// One `.icon` per source file, named by its stem, plus a Rust module named after `set` in
/// snake case. Sources are taken in sorted order so that regenerating an unchanged directory
/// produces an unchanged module.
///
/// Returns the file stems that were baked, in the order they were written.
///
/// # Errors
///
/// If the directory cannot be read, holds no `.svg`, or any one of them has no geometry to bake.
/// Nothing is written unless every field bakes, so a failed run leaves the output as it was.
pub fn bake_dir(svgs: &Path, out: &Path, spec: Bake, set: &str) -> Result<Vec<String>, String> {
    let mut sources: Vec<_> = fs::read_dir(svgs)
        .map_err(|e| format!("reading {}: {e}", svgs.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "svg"))
        .collect();
    sources.sort();

    if sources.is_empty() {
        return Err(format!("no .svg files in {}", svgs.display()));
    }

    // Every field is baked before anything is written, so a source that cannot bake fails the
    // run rather than leaving a set that names files which are not there.
    let mut baked = Vec::with_capacity(sources.len());
    for source in &sources {
        let stem = source
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("{}: filename is not utf-8", source.display()))?
            .to_string();
        let svg = fs::read(source).map_err(|e| format!("reading {}: {e}", source.display()))?;
        let field = bake(&svg, spec).map_err(|e| format!("{stem}: {e}"))?;
        baked.push((stem, field));
    }

    fs::create_dir_all(out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let mut stems = Vec::with_capacity(baked.len());
    for (stem, field) in &baked {
        let path = out.join(format!("{stem}.icon"));
        fs::write(&path, &field.rgba).map_err(|e| format!("writing {}: {e}", path.display()))?;
        stems.push(stem.clone());
    }

    let module = out.join(format!("{}.rs", naming::snake_case(set)));
    let generated = generate(&stems, spec, set)?;
    fs::write(&module, generated).map_err(|e| format!("writing {}: {e}", module.display()))?;
    Ok(stems)
}
