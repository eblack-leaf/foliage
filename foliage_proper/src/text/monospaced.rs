use crate::Resource;

#[derive(Resource)]
pub(crate) struct MonospacedFont(pub(crate) fontdue::Font);
impl MonospacedFont {
    pub(crate) fn new(opt_scale: u32) -> Self {
        Self(
            fontdue::Font::from_bytes(
                include_bytes!("JetBrainsMono-Medium.ttf").as_slice(),
                fontdue::FontSettings {
                    scale: opt_scale as f32,
                    ..fontdue::FontSettings::default()
                },
            )
                .expect("font"),
        )
    }
}
#[test]
fn metrics() {
    use crate::FontSize;
    let mut font = MonospacedFont::new(FontSize::default().value);
    let metrics = font.0.metrics('a', FontSize::default().value as f32);
    let line_metrics = font
        .0
        .horizontal_line_metrics(FontSize::default().value as f32);
    println!("{:?}\n{:?}", metrics, line_metrics);
}
