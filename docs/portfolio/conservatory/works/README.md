# Works: real pictures for the conservatory

Every work in *latent flora* is grown from a seed by default (`Art::Grown` in
`src/bin/conservatory/works.rs`). Any of them can hang a real picture instead.

1. Put a PNG or JPEG in this folder, for example `moss-liturgy.png`. Square pictures work best, since the
   hall and the viewing room crop to a square.
2. In `works.rs`, change that work's `art: Art::Grown` to `art: Art::File("moss-liturgy.png")`.

That is all. Natively, the picture is read from this folder. On the web, trunk copies this folder next to
the page (`rel="copy-dir"` in `web/conservatory/index.html`), and it is fetched from
`/foliage/portfolio/conservatory/works/<file>`.

If the file is missing, the work quietly falls back to being grown, so a typo costs the picture, not the
page. A real picture is shown as it is: there is no replay, latent walk or variation for it.
