//! The command line over the baker.

use std::fs;
use std::path::PathBuf;

use clap::Parser;
use foliage_icons::{Bake, bake_dir, preview};

/// Bakes SVGs into the fields foliage draws marks from.
#[derive(Parser)]
#[command(name = "foliage-icons", version)]
enum Cli {
    /// Bake every SVG in a directory, and write the set that registers them.
    Bake {
        /// The directory of `.svg` sources. One file is one mark.
        #[arg(long)]
        svg: PathBuf,
        /// Where the `.icon` files and the generated module are written.
        #[arg(long)]
        out: PathBuf,
        /// What to call the generated `Marks` set.
        #[arg(long, default_value = "Icons")]
        marks: String,
        /// Texels per side of the square field. One field serves every on-screen size, so bake
        /// generously: 32 to 64 keeps a small mark crisp.
        #[arg(long, default_value_t = 48)]
        side: u32,
        /// How many texels the distance spread covers. Two to four balances sharpness at small
        /// sizes against smoothness at large.
        #[arg(long, default_value_t = 3.0)]
        range: f32,
    },
    /// Render a baked field to a PNG, sampling exactly as the shader does, to judge its quality
    /// without running an application.
    Preview {
        /// The `.icon` file to render.
        #[arg(long)]
        icon: PathBuf,
        /// On-screen size in pixels, square.
        #[arg(long)]
        size: u32,
        /// The spread the field was baked with.
        #[arg(long, default_value_t = 3.0)]
        range: f32,
        /// Where the PNG is written.
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> Result<(), String> {
    match Cli::parse() {
        Cli::Bake {
            svg,
            out,
            marks,
            side,
            range,
        } => {
            if side < 8 {
                return Err("--side must be at least 8".to_string());
            }
            if !(range > 0.0) || range * 2.0 >= side as f32 {
                return Err(format!(
                    "--range must be above zero and leave an interior at --side {side}"
                ));
            }
            let stems = bake_dir(&svg, &out, Bake::new(side, range), &marks)?;
            for stem in &stems {
                println!("{stem}.icon");
            }
            println!(
                "baked {} mark{} into {}",
                stems.len(),
                match stems.len() {
                    1 => "",
                    _ => "s",
                },
                out.display()
            );
            Ok(())
        }
        Cli::Preview {
            icon,
            size,
            range,
            out,
        } => {
            let field = fs::read(&icon).map_err(|e| format!("reading {}: {e}", icon.display()))?;
            let png = preview(&field, size, range)?;
            fs::write(&out, png).map_err(|e| format!("writing {}: {e}", out.display()))?;
            println!("{} at {size}px -> {}", icon.display(), out.display());
            Ok(())
        }
    }
}
