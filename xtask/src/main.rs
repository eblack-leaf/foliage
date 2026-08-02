use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Repo tasks that used to be loose `.sh` files at the root. Rust rather than shell so they
/// run the same on Windows as on unix -- the external tools they drive (`trunk`, `mdbook`,
/// `git-cliff`) are all `cargo install`-able everywhere, so nothing here is unix-only.
///
/// Run as `cargo xtask <command>`, via the alias in `.cargo/config.toml`.
#[derive(Parser)]
#[command(name = "xtask")]
enum Cli {
    /// Build the site into `docs/` for GitHub Pages.
    ///
    /// Clears `docs/` first: `filehash = true` means every build emits differently-named
    /// bundles, which would otherwise pile up forever. That also removes `docs/book` and
    /// `docs/api` -- run `web` to rebuild all three together.
    Site,
    /// Serve the app locally with auto-reload (`trunk serve`, debug profile).
    Serve,
    /// Build the mdbook into `docs/book`.
    Book,
    /// Build the rustdoc API reference into `docs/api`.
    Api,
    /// Regenerate `CHANGELOG.md` from git history.
    Changelog,
    /// Build everything `docs/` holds: `site`, then `book`, then `api`. Order matters --
    /// `site` clears `docs/`, so the other two have to repopulate it afterwards.
    Web,
}

fn main() -> Result<(), String> {
    match Cli::parse() {
        Cli::Site => site(),
        Cli::Serve => serve(),
        Cli::Book => book(),
        Cli::Api => api(),
        Cli::Changelog => changelog(),
        Cli::Web => {
            site()?;
            book()?;
            api()
        }
    }
}

fn site() -> Result<(), String> {
    let root = root();
    let app = root.join("application");
    let dist = app.join("dist");
    let docs = root.join("docs");

    // Everything destructive happens *after* this succeeds. The shell version this replaced
    // had no `set -e`, so a failed build still fell through to wiping `docs/`.
    run("trunk", &["build", "--release"], &app)?;

    clear_dir(&docs)?;
    copy_dir(&dist, &docs)?;
    rm_rf(&dist)?;
    println!("built site -> {}", docs.display());
    Ok(())
}

fn serve() -> Result<(), String> {
    run("trunk", &["serve"], &root().join("application"))
}

fn book() -> Result<(), String> {
    let root = root();
    let out = root.join("docs/book");
    run("mdbook", &["build", "foliage/book"], &root)?;
    clear_dir(&out)?;
    copy_dir(&root.join("foliage/book/dist"), &out)?;
    println!("built book -> {}", out.display());
    Ok(())
}

fn api() -> Result<(), String> {
    let root = root();
    let target = root.join("target/doc");
    let out = root.join("docs/api");

    // rustdoc merges into `target/doc` rather than replacing it, so items deleted since the
    // last build would otherwise linger in the copied output.
    rm_rf(&target)?;
    run("cargo", &["doc", "--no-deps", "-p", "foliage"], &root)?;
    clear_dir(&out)?;
    copy_dir(&target, &out)?;

    // search.index powers the search box (which stops working without it) and type.impl the
    // "Implementations on Foreign Types" sections -- ~1M of index for a crate this size, judged
    // not worth keeping in git.
    rm_rf(&out.join("search.index"))?;
    rm_rf(&out.join("type.impl"))?;
    // cargo's lock for `target/doc`, not output. The `cp -r target/doc/*` this replaced never
    // copied it because a shell glob skips dotfiles; `copy_dir` walks the directory instead.
    rm_rf(&out.join(".lock"))?;

    let unwrapped = fix_links(&out)?;
    println!("unwrapped {unwrapped} unresolvable links");

    println!("built api docs -> {}", out.display());
    Ok(())
}

/// Repair the links rustdoc itself leaves pointing at nothing: `--no-deps` means the crate's
/// own dependencies (`tracing`, `winit`, `bytemuck`, ...) get no doc pages of their own to link
/// to, so an intra-doc link to one of their items (`href="crate::Span"`,
/// `href="super::Subscriber"`) is emitted verbatim, unresolved. Nothing to point those at, so
/// the `<a>` is dropped and its text kept -- the test is whether the target file exists on disk,
/// so this stays correct if a later trim deletes something new.
fn fix_links(out: &Path) -> Result<usize, String> {
    let mut unwrapped = 0;
    for page in html_files(out)? {
        let source = fs::read_to_string(&page).map_err(|e| format!("{}: {e}", page.display()))?;
        let (fixed, dropped) = unwrap_dead_links(&source, page.parent().expect("file has a dir"));
        unwrapped += dropped;
        fs::write(&page, fixed).map_err(|e| format!("writing {}: {e}", page.display()))?;
    }
    Ok(unwrapped)
}

/// Replace every `<a href="...">text</a>` whose target does not exist with just `text`.
/// Absolute, `mailto:` and same-page (`#...`) hrefs are left alone; so is any `<a>` missing a
/// closing tag, which would mean the input is not the rustdoc output this expects.
fn unwrap_dead_links(page: &str, dir: &Path) -> (String, usize) {
    let mut fixed = String::with_capacity(page.len());
    let mut rest = page;
    let mut dropped = 0;
    while let Some(start) = rest.find("<a ") {
        let (before, from_tag) = rest.split_at(start);
        fixed.push_str(before);
        let Some(tag_end) = from_tag.find('>') else {
            break;
        };
        let (tag, inner) = from_tag.split_at(tag_end + 1);
        match attribute(tag, "href").filter(|href| is_dead(href, dir)) {
            Some(_) => match inner.find("</a>") {
                Some(close) => {
                    fixed.push_str(&inner[..close]);
                    rest = &inner[close + "</a>".len()..];
                    dropped += 1;
                }
                None => break,
            },
            None => {
                fixed.push_str(tag);
                rest = inner;
            }
        }
    }
    fixed.push_str(rest);
    (fixed, dropped)
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!("{name}=\""))?.1;
    Some(value.split_once('"')?.0)
}

fn is_dead(href: &str, dir: &Path) -> bool {
    if href.starts_with('#') || href.contains("://") || href.starts_with("mailto:") {
        return false;
    }
    // Strip the fragment: it names an anchor within the target page, not a path.
    let path = href.split('#').next().unwrap_or_default();
    !path.is_empty() && !dir.join(path).exists()
}

fn html_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|e| format!("reading {}: {e}", dir.display()))?
            .path();
        if path.is_dir() {
            found.extend(html_files(&path)?);
        } else if path.extension().is_some_and(|e| e == "html") {
            found.push(path);
        }
    }
    Ok(found)
}

fn changelog() -> Result<(), String> {
    // Pinned to v1.0.0.. on purpose: the pre-1.0 history is churn nobody needs a changelog for.
    // Invoked as the `git-cliff` binary rather than `git cliff` so a missing install reports
    // itself instead of surfacing as a git "not a git command" message.
    run("git-cliff", &["v1.0.0..", "-o", "CHANGELOG.md"], &root())
}

/// The workspace root -- this crate's manifest dir, one level up. Taken from the environment
/// rather than the current directory so every task works from anywhere in the repo.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ always has a parent")
        .to_path_buf()
}

fn run(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    println!("$ {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .status()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                format!("`{program}` not found -- install it with `cargo install {program}`")
            }
            _ => format!("running {program}: {e}"),
        })?;
    if !status.success() {
        return Err(format!("{program} failed with {status}"));
    }
    Ok(())
}

/// Remove a file or directory if it exists. Missing is success -- these all run against paths
/// that may or may not be there yet.
fn rm_rf(path: &Path) -> Result<(), String> {
    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else if path.exists() {
        fs::remove_file(path)
    } else {
        return Ok(());
    };
    result.map_err(|e| format!("removing {}: {e}", path.display()))
}

/// Empty a directory but keep the directory itself, creating it if absent.
fn clear_dir(path: &Path) -> Result<(), String> {
    rm_rf(path)?;
    fs::create_dir_all(path).map_err(|e| format!("creating {}: {e}", path.display()))
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("creating {}: {e}", dst.display()))?;
    let entries = fs::read_dir(src).map_err(|e| format!("reading {}: {e}", src.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("reading {}: {e}", src.display()))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)
                .map_err(|e| format!("copying {} -> {}: {e}", from.display(), to.display()))?;
        }
    }
    Ok(())
}
