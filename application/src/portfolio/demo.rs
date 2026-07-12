use foliage::{
    Animation, Color, EcsExtension, Elevation, Entity, FontSize, Grid, GridExt, LineConstraint,
    Location, Opacity, Sequence, Sprout, Text, TextInput, Tree,
};

/// Stands in for the "Artist Blog" portfolio item until that app is built.
pub(crate) fn build(tree: &mut Tree, app: Entity) {
    Sequence::new(tree).animate(
        Animation::new(Opacity::new(1.0))
            .start(1000)
            .finish(1500)
            .targeting(app),
    );
    tree.branch(
        app,
        Text::new("composites")
            .size(FontSize::new(24))
            .color(Color::gray(400))
            .at(Location::new().xs(
                1.col().as_left().with(12.col().as_right()),
                1.row().as_top().with(1.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
    tree.branch(
        app,
        TextInput::new()
            .line_constraint(LineConstraint::Multiple)
            .hint_text("multiline input...")
            .foreground(Color::gray(200))
            .background(Color::gray(900))
            .accent(Color::green(600))
            .font_size(FontSize::new(16))
            .at(Location::new().xs(
                1.col().as_left().with(6.col().as_right()),
                2.row().as_top().with(5.row().as_bottom()),
            ))
            .elevate(Elevation::up(1)),
    );
}
