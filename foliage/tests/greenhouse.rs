//! The greenhouse, from outside the crate: an app grown, pressed, timed and pictured with nothing
//! but the public surface -- which is the whole of what an app's own tests can reach.

use foliage::{
    Area, Boxed, Color, Greenhouse, Grove, Grow, Leaf, Location, Motion, Palette, Panel, Place,
    Pollen, Position, Root, Source, Stem, Timing, Tween, left, top,
};

/// A square that counts its clicks, and a fade an app can start.
struct Square {
    square: Leaf,
    clicks: u32,
    hovered: bool,
    fade: Option<Tween>,
    faded: bool,
}

const RED: Color = Color::rgb(1.0, 0.0, 0.0);
const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);

impl Root for Square {
    fn take_root(grove: &mut Grove) -> Self {
        let square =
            grove.plant(Panel::new().color(RED).interactive().at(
                Location::new().xs(left(20.px()).width(40.px()), top(20.px()).height(40.px())),
            ));
        Square {
            square,
            clicks: 0,
            hovered: false,
            fade: None,
            faded: false,
        }
    }

    fn frame(&mut self, _grove: &mut Grove, pollen: Pollen) {
        if pollen.clicked(self.square) {
            self.clicks += 1;
        }
        if pollen.hovered(self.square) {
            self.hovered = true;
        }
        if pollen.unhovered(self.square) {
            self.hovered = false;
        }
        if self.fade.is_some_and(|fade| pollen.finished(fade)) {
            self.faded = true;
        }
    }
}

fn house() -> Greenhouse<Square> {
    let mut house = Greenhouse::<Square>::new(Area::new(100.0, 80.0));
    house.frame();
    house
}

#[test]
fn a_tap_reaches_the_app() {
    let mut house = house();
    house.tap(Position::new(40.0, 40.0));
    house.frame();
    house.frame();
    assert_eq!(house.root().clicks, 1);
    // Off the square, nothing.
    house.tap(Position::new(90.0, 70.0));
    house.settle();
    assert_eq!(house.root().clicks, 1);
}

#[test]
fn where_the_pointer_rests_is_hovered() {
    let mut house = house();
    house.point(Position::new(30.0, 30.0));
    house.settle();
    assert!(house.root().hovered);
    assert_eq!(house.grove().hovered(), Some(house.root().square));
    house.leave();
    house.settle();
    assert!(!house.root().hovered);
}

#[test]
fn settling_runs_a_motion_to_its_end() {
    let mut house = house();
    let square = house.root().square;
    let fade = house
        .grove_mut()
        .animate(square, Motion::Opacity(0.0), Timing::ms(300));
    house.root_mut().fade = Some(fade);
    let frames = house.settle();
    // Sixteen milliseconds apart, so about twenty of them -- and the report a frame later.
    assert!((18..=24).contains(&frames), "{frames} frames");
    assert!(house.root().faded);
}

/// A picture is the frame as drawn. A machine with no adapter at all has nothing to draw with, and
/// says so rather than failing.
#[test]
fn a_picture_is_what_was_drawn() {
    let mut house = house();
    let Some(picture) = house.render() else {
        eprintln!("no adapter: nothing to draw with");
        return;
    };
    assert_eq!((picture.width(), picture.height()), (100, 80));
    let close = |a: Color, b: Color| {
        (a.red - b.red).abs() < 0.02
            && (a.green - b.green).abs() < 0.02
            && (a.blue - b.blue).abs() < 0.02
    };
    let inside = picture
        .at(Position::new(40.0, 40.0))
        .expect("on the picture");
    assert!(close(inside, RED), "{inside:?}");
    let ground = house.grove().scheme().color(Palette::Surface);
    let outside = picture
        .at(Position::new(90.0, 70.0))
        .expect("on the picture");
    assert!(close(outside, ground), "{outside:?} against {ground:?}");
    assert!(picture.at(Position::new(100.0, 10.0)).is_none());

    // Every frame after the first picture reaches the device as it runs.
    let square = house.root().square;
    house.grove_mut().color(square, BLUE);
    house.frame();
    let picture = house.render().expect("the same device again");
    let inside = picture
        .at(Position::new(40.0, 40.0))
        .expect("on the picture");
    assert!(close(inside, BLUE), "{inside:?}");

    // A resize is followed by the picture.
    house.resize(Area::new(64.0, 48.0));
    house.frame();
    let picture = house.render().expect("the same device again");
    assert_eq!((picture.width(), picture.height()), (64, 48));
    assert_eq!(&picture.png()[1..4], b"PNG");
}

/// Two squares overlapping, the blue one in front, in a card at half opacity -- drawn together
/// where `TOGETHER`, each on its own where not.
struct Card<const TOGETHER: bool> {
    card: Leaf,
}

impl<const TOGETHER: bool> Root for Card<TOGETHER> {
    fn take_root(grove: &mut Grove) -> Self {
        let mut card = Stem::new()
            .opacity(0.5)
            .at(Location::new().xs(left(0.px()).width(100.px()), top(0.px()).height(80.px())));
        if TOGETHER {
            card = card.together();
        }
        let card = grove.plant(card);
        grove.branch(
            card,
            Panel::new()
                .color(RED)
                .at(Location::new().xs(left(10.px()).width(40.px()), top(10.px()).height(40.px()))),
        );
        grove.branch(
            card,
            Panel::new()
                .color(BLUE)
                .at(Location::new().xs(left(30.px()).width(40.px()), top(30.px()).height(40.px()))),
        );
        Card { card }
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {}
}

fn near(a: Color, b: Color) -> bool {
    (a.red - b.red).abs() < 0.02
        && (a.green - b.green).abs() < 0.02
        && (a.blue - b.blue).abs() < 0.02
}

/// What fades together is faded whole: where its parts overlap, the one behind does not show
/// through the one in front.
#[test]
fn what_is_drawn_together_fades_whole() {
    let mut house = Greenhouse::<Card<true>>::new(Area::new(100.0, 80.0));
    house.frame();
    let Some(picture) = house.render() else {
        eprintln!("no adapter: nothing to draw with");
        return;
    };
    let ground = house.grove().scheme().color(Palette::Surface);
    let half = |over: Color| {
        Color::rgb(
            (over.red + ground.red) / 2.0,
            (over.green + ground.green) / 2.0,
            (over.blue + ground.blue) / 2.0,
        )
    };
    // Each alone, half over the ground.
    let red = picture.at(Position::new(15.0, 15.0)).unwrap();
    assert!(near(red, half(RED)), "{red:?}");
    let blue = picture.at(Position::new(65.0, 65.0)).unwrap();
    assert!(near(blue, half(BLUE)), "{blue:?}");
    // Where they overlap, the blue in front and nothing of the red behind it.
    let both = picture.at(Position::new(40.0, 40.0)).unwrap();
    assert!(near(both, half(BLUE)), "{both:?} against {:?}", half(BLUE));
    // And off them, the ground untouched.
    let off = picture.at(Position::new(90.0, 5.0)).unwrap();
    assert!(near(off, ground), "{off:?}");

    // Brought up to full, nothing is drawn apart, and the front square covers the one behind.
    let card = house.root().card;
    house.grove_mut().opacity(card, 1.0);
    house.frame();
    let picture = house.render().expect("the same device again");
    let both = picture.at(Position::new(40.0, 40.0)).unwrap();
    assert!(near(both, BLUE), "{both:?}");

    // And faded again, the picture comes back.
    house.grove_mut().opacity(card, 0.25);
    house.frame();
    let picture = house.render().expect("the same device again");
    let both = picture.at(Position::new(40.0, 40.0)).unwrap();
    let quarter = Color::rgb(
        BLUE.red * 0.25 + ground.red * 0.75,
        BLUE.green * 0.25 + ground.green * 0.75,
        BLUE.blue * 0.25 + ground.blue * 0.75,
    );
    assert!(near(both, quarter), "{both:?} against {quarter:?}");

    // Resized while faded, the picture it is drawn into is made again at the new size, and the
    // card is still in the frame rather than dropped for want of one.
    house.resize(Area::new(120.0, 90.0));
    house.frame();
    let picture = house.render().expect("the same device again");
    assert_eq!((picture.width(), picture.height()), (120, 90));
    let both = picture.at(Position::new(40.0, 40.0)).unwrap();
    assert!(near(both, quarter), "{both:?} against {quarter:?}");
}

/// Without it, each element fades on its own and the red shows through where they overlap -- the
/// case `together` is for.
#[test]
fn what_is_faded_apart_shows_through() {
    let mut house = Greenhouse::<Card<false>>::new(Area::new(100.0, 80.0));
    house.frame();
    let Some(picture) = house.render() else {
        return;
    };
    let both = picture.at(Position::new(40.0, 40.0)).unwrap();
    assert!(both.red > 0.15, "{both:?}");
}

/// A square turned an eighth, and a card scaled to half with a square on it.
struct Posed;

impl Root for Posed {
    fn take_root(grove: &mut Grove) -> Self {
        // 40 across, its middle at (50, 40).
        grove.plant(
            Panel::new()
                .color(RED)
                .turn(core::f32::consts::FRAC_PI_4)
                .at(Location::new().xs(left(30.px()).width(40.px()), top(20.px()).height(40.px()))),
        );
        // A card filling the right of the window, scaled to half about its middle (150, 40):
        // the square on it, at its top-left quarter, lands nearer that middle.
        let card =
            grove.plant(Stem::new().scale(0.5).at(
                Location::new().xs(left(100.px()).width(100.px()), top(0.px()).height(80.px())),
            ));
        grove.branch(
            card,
            Panel::new()
                .color(BLUE)
                .at(Location::new().xs(left(0.px()).width(50.px()), top(0.px()).height(40.px()))),
        );
        Posed
    }

    fn frame(&mut self, _grove: &mut Grove, _pollen: Pollen) {}
}

/// What is turned or scaled is drawn turned or scaled, by the renderer itself.
#[test]
fn a_pose_is_drawn() {
    let mut house = Greenhouse::<Posed>::new(Area::new(200.0, 80.0));
    house.frame();
    let Some(picture) = house.render() else {
        return;
    };
    let ground = house.grove().scheme().color(Palette::Surface);
    let at = |x: f32, y: f32| picture.at(Position::new(x, y)).unwrap();
    // Turned, it reaches past its upright edge along its middle, and leaves its corner.
    assert!(near(at(75.0, 40.0), RED), "{:?}", at(75.0, 40.0));
    assert!(near(at(32.0, 22.0), ground), "{:?}", at(32.0, 22.0));
    // Scaled to half about (150, 40): the square's box 100..150 by 0..40 draws at 125..150 by
    // 20..40.
    assert!(near(at(130.0, 30.0), BLUE), "{:?}", at(130.0, 30.0));
    assert!(near(at(110.0, 10.0), ground), "{:?}", at(110.0, 10.0));
}
