//! ONE-OFF ASSEMBLY. The rule: `tree.branch(parent, ..)` + locals. No composite closures, no
//! tuple returns, no Handle types, ever — an entity you need later is a local, an entity
//! you don't is never named. music_player.rs already proved this style is the readable
//! one; it is now the ONLY one-off style (`tree.composite()` and home.rs's nested
//! 12-tuple closures are deleted).

use super::zoo::*;
use foliage::*;

#[derive(Resource, Default)]
struct UserSettings {
    volume: f32,
    quality: usize,
}

fn settings_screen(tree: &mut Tree) {
    let root = tree.leaf(
        Leaf::sprout()
            .at(Location::new().xs(
                0.pct().as_left().with(100.pct().as_right()),
                0.pct().as_top().with(100.pct().as_bottom()),
            ))
            .elevate(Elevation::abs(0))
            .with(Grid::new(12.col().gap(8), 48.px().gap(12))),
    );
    tree.branch(
        root,
        Text::new("Settings")
            .size(FontSize::new(24))
            .at(/* row 1 */)
            .elevate(Elevation::up(1)),
    );

    // library widgets and the user's own widgets spawn through the same chain —
    // a CardView or a Slider, no difference from here
    let volume = tree.branch(root, Slider::new().value(0.7).at(/* row 2 */).elevate(Elevation::up(1)));
    let muted = tree.branch(root, Checkbox::new().at(/* row 3 */).elevate(Elevation::up(1)));
    let quality = tree.branch(root, Dropdown::new()
            .options(["Low", "Medium", "High"])
            .at(/* row 4 */)
            .elevate(Elevation::up(1)),
    );

    // cross-widget behavior is end-user glue, written in widget vocabulary:
    tree.subscribe(muted, move |t: Trigger<Toggled>, mut tree: Tree| {
        if t.event().checked {
            tree.disable(volume);
        } else {
            tree.enable(volume);
        }
    });
    tree.subscribe(
        volume,
        |t: Trigger<ValueChanged>, mut settings: ResMut<UserSettings>| {
            settings.volume = t.event().value;
        },
    );
    tree.subscribe(
        quality,
        |t: Trigger<SelectionChanged>, mut settings: ResMut<UserSettings>| {
            settings.quality = t.event().index;
        },
    );

    // teardown of the whole screen, later, from anywhere: tree.remove(root) —
    // every widget above stems from root, the cascade does the rest.
}
