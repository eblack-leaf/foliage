mod codegen;
mod msdf;
mod naming;
mod preview;

use clap::Parser;
use codegen::{CodegenConfig, IconEntry};
use msdf::generate_field;
use std::fs;
use std::path::PathBuf;

/// SVG -> foliage `.icon` MTSDF format generator, plus registration codegen.
#[derive(Parser)]
#[command(name = "foliage_icons")]
enum Cli {
    /// Bake every SVG in a directory into a resolution-independent `.icon` MTSDF field + a
    /// generated registration module.
    Gen {
        /// Directory of `.svg` source files (one file = one icon).
        #[arg(long)]
        svg: PathBuf,
        /// Output directory for `.icon` files + the generated `.rs` module.
        #[arg(long)]
        out: PathBuf,
        /// Square MTSDF field resolution (texels per side). One field serves every on-screen
        /// size; bake generously (32-64) so small sizes stay crisp.
        #[arg(long, default_value_t = 48)]
        field_size: u32,
        /// Distance-field spread, in texels. ~2-4 balances small-size sharpness vs. large-size
        /// smoothness.
        #[arg(long, default_value_t = 3.0)]
        px_range: f32,
        /// Name of the generated `#[icon_handle]` enum.
        #[arg(long, default_value = "IconHandles")]
        enum_name: String,
    },
    /// Render a baked `.icon` field to a PNG at an on-screen size, sampling exactly as the
    /// shader does, to judge field quality without running an application.
    Preview {
        /// The `.icon` file to render.
        #[arg(long)]
        icon: PathBuf,
        /// On-screen size in pixels (square).
        #[arg(long)]
        size: u32,
        /// Distance-field spread the icon was baked with.
        #[arg(long, default_value_t = 3.0)]
        px_range: f32,
        /// Output PNG path.
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> Result<(), String> {
    match Cli::parse() {
        Cli::Gen {
            svg,
            out,
            field_size,
            px_range,
            enum_name,
        } => generate(svg, out, field_size, px_range, enum_name),
        Cli::Preview {
            icon,
            size,
            px_range,
            out,
        } => preview::render(&icon, size, px_range, &out),
    }
}

fn generate(
    svg_dir: PathBuf,
    out_dir: PathBuf,
    field_size: u32,
    px_range: f32,
    enum_name: String,
) -> Result<(), String> {
    if field_size < 8 {
        return Err("--field-size must be at least 8".to_string());
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
        let field = generate_field(&svg_bytes, field_size, px_range as f64)
            .map_err(|e| format!("{stem}: {e}"))?;

        let icon_path = out_dir.join(format!("{stem}.icon"));
        fs::write(&icon_path, &field.rgba)
            .map_err(|e| format!("writing {}: {e}", icon_path.display()))?;

        println!(
            "{stem}.icon: {} bytes ({}x{} MTSDF, px_range {})",
            field.rgba.len(),
            field.size,
            field.size,
            field.px_range,
        );

        entries.push(IconEntry {
            variant,
            file_stem: stem,
        });
    }

    let cfg = CodegenConfig {
        enum_name,
        field_size,
        px_range,
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
