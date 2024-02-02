# Entry

### Setup

First you will need two crates. One to serve as the entry point, and one for your application.

```shell
cargo new entry --bin
cargo new example --lib
```

The `entry` crate will need to be configured as a dynamic library.

```toml
[lib]
name = "entry"
path = "<path-to>/lib.rs"
crate-type = ["cdylib"]
```

Android will need a hook in the `cdylib`.

```rust
// android app hook
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: foliage::AndroidApp) {
    example::entry(foliage::AndroidInterface::new(app));
}
```

For web to have a background thread to handle async requests, a web-worker must be configured.

```toml
[[bin]]
name = "worker"
path = "<path-to>/worker.rs"
```
This worker will need to be started in this linked binary.
```rust
#[allow(unused)]
fn main() {
    foliage::workflow::start_web_worker::<example::Engen>();
}
```

For using `trunk` as a web packaging tool, add an `index.html` to the entry crate root.

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8"/>
    <meta content="width=device-width, viewport-fit=cover, initial-scale=1.0" name="viewport"/>
    <style type="text/css">
        html, body {
            margin: 0px;
            border: 0px;
            padding: 0px;
            height: 100%;
            background: #111111;
        }
    </style>
    <link data-bin="entry" data-trunk data-type="main" data-wasm-opt="z" href="Cargo.toml" rel="rust"/>
    <link data-trunk rel="rust" href="Cargo.toml" data-wasm-opt="z" data-bin="worker" data-type="worker" />
    <link data-trunk rel="copy-dir" href="../example/assets"/>
</head>
<body>
</body>
</html>
```

This will remove padding to let the canvas element from winit have the full screen.
It also links the binaries to the web entry point in `trunk`. It also optionally adds an
asset directory `../example/assets` which will copy any files at that location into the 
served directory to be fetched. All of the configuration details for `trunk` can be found on
[trunk.rs](https://trunkrs.dev/)