mod codegen;
mod naming;
mod rasterize;

use clap::Parser;
use codegen::{CodegenConfig, IconEntry};
use rasterize::{concat_levels, rasterize_svg};
use std::fs;
use std::path::PathBuf;

/// SVG -> foliage `.icon` binary format generator, plus registration codegen.
#[derive(Parser)]
#[command(name = "foliage_icons")]
enum Cli {
    /// Rasterize every SVG in a directory into `.icon` files + a generated registration module.
    Gen {
        /// Directory of `.svg` source files (one file = one icon).
        #[arg(long)]
        svg: PathBuf,
        /// Output directory for `.icon` files + the generated `.rs` module.
        #[arg(long)]
        out: PathBuf,
        /// Logical on-screen render size, in pixels (the smallest/sharpest-at-1x mip level).
        #[arg(long, default_value_t = 24)]
        size: u32,
        /// Mip level count. Buckets are always a doubling chain (`size << level`), so this
        /// is the only knob -- e.g. size=24 mips=3 reproduces today's 96/48/24 chain exactly.
        #[arg(long, default_value_t = 3)]
        mips: u32,
        /// Name of the generated `#[icon_handle]` enum.
        #[arg(long, default_value = "IconHandles")]
        enum_name: String,
    },
}

fn main() -> Result<(), String> {
    match Cli::parse() {
        Cli::Gen {
            svg,
            out,
            size,
            mips,
            enum_name,
        } => generate(svg, out, size, mips, enum_name),
    }
}

fn generate(
    svg_dir: PathBuf,
    out_dir: PathBuf,
    size: u32,
    mips: u32,
    enum_name: String,
) -> Result<(), String> {
    if mips == 0 {
        return Err("--mips must be at least 1".to_string());
    }
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let mut svg_files: Vec<PathBuf> = fs::read_dir(&svg_dir)
        .map_err(|e| format!("reading {}: {e}", svg_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("svg"))
        .collect();
    svg_files.sort();

    if svg_files.is_empty() {
        return Err(format!("no .svg files found in {}", svg_dir.display()));
    }

    let mut entries = Vec::with_capacity(svg_files.len());
    for svg_path in &svg_files {
        let stem = svg_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("invalid filename: {}", svg_path.display()))?
            .to_string();
        let variant = naming::pascal_case(&stem);

        let svg_bytes = fs::read(svg_path).map_err(|e| format!("reading {stem}.svg: {e}"))?;
        let levels = rasterize_svg(&svg_bytes, size, mips).map_err(|e| format!("{stem}: {e}"))?;
        let bytes = concat_levels(&levels);

        let icon_path = out_dir.join(format!("{stem}.icon"));
        fs::write(&icon_path, &bytes)
            .map_err(|e| format!("writing {}: {e}", icon_path.display()))?;

        println!(
            "{stem}.icon: {} bytes ({mips} levels, {}px..{}px)",
            bytes.len(),
            levels.last().map(|l| l.px).unwrap_or(0),
            levels.first().map(|l| l.px).unwrap_or(0),
        );

        entries.push(IconEntry {
            variant,
            file_stem: stem,
        });
    }

    let cfg = CodegenConfig {
        enum_name,
        size,
        mips,
        texture_scale: size << (mips - 1),
    };
    let generated = codegen::generate(&entries, &cfg);
    let generated_path = out_dir.join("generated.rs");
    fs::write(&generated_path, generated)
        .map_err(|e| format!("writing {}: {e}", generated_path.display()))?;

    println!(
        "generated {} ({} icons)",
        generated_path.display(),
        entries.len()
    );
    Ok(())
}
