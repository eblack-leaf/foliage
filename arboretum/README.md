# arboretum

A portfolio of three apps grown on foliage and lichen. Each is a binary of this crate, built on its
own with trunk and served under the project page at `/foliage/portfolio/<app>/`. None of them has a
real backend: each has an `api.rs` of plain functions over fixtures, answered through a courier that
adds latency, so the flows show waiting, failure and confirmation the way a real service would. A
real backend would replace `api.rs` and nothing else.

| app | what it is | its face |
|---|---|---|
| `conservatory` | *latent flora*: an exhibition of twelve AI-grown works in four rooms. The page comes up out of a dark, shaded wall. Each work develops as a generator's preview does, dim and soft first, then lit and sharp. There is a viewing room, a studio to grow your own work, and a visitors' wall. | Major Mono Display |
| `orchard` | *bits & bytes*: a boutique of digital goods (palettes, shaders, type specimens, sound packs), each weighed to the byte and printed on its label. There is a bag, a four-step till, and a receipt sealed in wax. | Xanh Mono |
| `trellis` | One drawing sheet for the people, meetings and specs of a launch. Nodes are dragged and wired. Clashes are marked up in red pencil and fixed through the courier. The sheet is read through instruments laid over it. | IBM Plex Mono |

All three set their UI in the bundled JetBrains Mono. Each face is OFL, kept with its licence under
`assets/fonts/`.

## Run

```sh
cargo run -p arboretum --bin conservatory      # or orchard, trellis
cargo xtask serve conservatory                 # on the web, with reload: :8081, :8082, :8083
cargo xtask portfolio                          # all three, release, into docs/portfolio/ with the hub
```

`cargo xtask portfolio` builds every app before touching `docs/portfolio/`, so a failed build leaves
the last portfolio where it was. `cargo xtask web` builds the site, then the portfolio, then the
book and the API docs.

## Layout

- `src/lib.rs`: boot (title, app id, window size, tracing). Shared by the three apps.
- `src/sprout/`: parts grown beside the apps as candidates for lichen. Among them are `Hover`, `Dashes`, `Wire`, `Drawer`, `Roll`, `Stamp`, `Courier`, `Emerge`, `Print` and `Signpost`. Each follows lichen's rules: the caller places it, it takes input in one element only, and it reports what happened.
- `src/genart/`: procedural pictures and the soft-focus develop. These are pure functions with unit tests.
- `src/bin/<app>/`: each app's own module tree, so each wasm build compiles only its own code and fonts.
- `web/<app>/`: each app's trunk project. `web/index.html` is the hub that links them.
- `assets/works/`: real pictures for the conservatory. See [its README](assets/works/README.md).

## Checks

```sh
cargo test -p arboretum
cargo clippy -p arboretum --all-targets
cargo check -p arboretum --target wasm32-unknown-unknown
```
