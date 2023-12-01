#[cfg(test)]
#[test]
fn test() {
    use crate::ginkgo::Ginkgo;
    use std::path::Path;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("panel").join("texture_resources");
    Ginkgo::png_to_cov(
        root.join("panel-ring-texture.png"),
        root.join("panel-ring-texture.cov")
    );
    Ginkgo::png_to_cov(
        root.join("panel-texture.png"),
        root.join("panel-texture.cov")
    );
}