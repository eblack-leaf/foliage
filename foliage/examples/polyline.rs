//! Plain vs. dashed `Polyline`, plus a `PolylineDrawProgress` loop mirroring the
//! `application` demo's own `drive_polyline_draw`. Run with
//! `cargo run --example polyline -p foliage`.

use foliage::{
    component, Color, DashPattern, EcsExtension, Elevation, Foliage, GridExt, Location, Polyline,
    PolylineDrawProgress, Position, Query, Res, Sprout, Time, Tree,
};

#[component]
struct DrawProgress {
    elapsed: f32,
}

const DRAW_CYCLE_SECS: f32 = 1.5;

fn drive(time: Res<Time>, mut query: Query<(foliage::Entity, &mut DrawProgress)>, mut tree: Tree) {
    for (entity, mut progress) in query.iter_mut() {
        progress.elapsed += time.frame_diff().as_secs_f32();
        let t = (progress.elapsed % DRAW_CYCLE_SECS) / DRAW_CYCLE_SECS;
        tree.write_to(entity, PolylineDrawProgress(t));
    }
}

fn main() {
    let mut foliage = Foliage::new();
    foliage.desktop_size((420, 160));

    let zigzag: Vec<Position<foliage::Logical>> = vec![
        (10, 90).into(),
        (50, 30).into(),
        (90, 90).into(),
        (130, 30).into(),
        (170, 70).into(),
    ];

    foliage.world.leaf(
        Polyline::new()
            .points(zigzag.clone())
            .weight(3)
            .color(Color::gray(300))
            .at(Location::new().xs(
                20.px().as_left().with(180.px().as_width()),
                20.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1))
            .with(DrawProgress { elapsed: 0.0 }),
    );

    foliage.world.leaf(
        Polyline::new()
            .points(zigzag)
            .weight(3)
            .color(Color::gray(300))
            .dash(DashPattern::new(10.0, 6.0))
            .at(Location::new().xs(
                220.px().as_left().with(180.px().as_width()),
                20.px().as_top().with(100.px().as_height()),
            ))
            .elevate(Elevation::up(1)),
    );

    foliage.user.add_systems(drive);
    foliage.photosynthesize();
}
