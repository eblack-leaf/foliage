use foliage::{
    Animation, Color, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt, HintText,
    LeafBuilder, LineConstraint, Location, Opacity, Primary, Secondary, Stem, Tertiary, Text,
    TextInput, Tree,
};

/// Stands in for the "Artist Blog" portfolio item until that app is built.
pub(crate) fn build(tree: &mut Tree, app: Entity) {
    let seq = tree.sequence();
    tree.animate(
        Animation::new(Opacity::new(1.0))
            .start(1000)
            .finish(1500)
            .during(seq)
            .targeting(app),
    );
    Text::new("composites")
        .size(FontSize::new(24))
        .color(Color::gray(400))
        .at(Location::new().xs(
            1.col().as_left().with(12.col().as_right()),
            1.row().as_top().with(1.row().as_bottom()),
        ))
        .elevate(Elevation::up(1))
        .stem(app)
        .spawn(tree);
    tree.leaf((
        TextInput::new(),
        LineConstraint::Multiple,
        HintText::new("multiline input..."),
        Primary(Color::gray(200)),
        Secondary(Color::gray(900)),
        Tertiary(Color::green(600)),
        FontSize::new(16),
        Location::new().xs(
            1.col().as_left().with(6.col().as_right()),
            2.row().as_top().with(5.row().as_bottom()),
        ),
        Stem::some(app),
        Elevation::up(1),
    ));
}
