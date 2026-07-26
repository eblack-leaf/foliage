use crate::morph::morph_in;
use crate::shadow::shadow_of;
use foliage::bevy_ecs::lifecycle::Insert;
use foliage::{
    Color, CurrentInteraction, Dragged, EcsExtension, Elevation, Entity, Grid, GridExt,
    InteractionListener, InteractionPropagation, InteractionShape, Leaf, Line, Location, Logical,
    OnClick, Opacity, Polygon, Query, Res, ScrollProgress, ScrollTo, Section, Sprout, Tree,
    Trigger,
};

/// Every knob-with-a-shadow scrollbar in this app is a heptagon -- narrower callers can
/// still get a plain triangle/square by passing a one-entry `knob_stages`.
const DEFAULT_KNOB_STAGES: &[(f32, f32)] = &[(7.0, 0.15)];

#[derive(Clone)]
pub struct ScrollbarStyle {
    /// px from the parent's own right edge to the hit-region's right edge.
    pub right_inset_px: i32,
    /// wider than the visual track -- an easier drag/tap target.
    pub hit_width_px: i32,
    pub track_top_pct: f32,
    pub track_bottom_pct: f32,
    pub track_weight: i32,
    pub track_color: Color,
    pub knob_size_px: i32,
    pub knob_color: Color,
    pub knob_shadow_color: Color,
    /// (sides, rounding) stages `morph_in` steps the knob through on spawn, after the
    /// starting triangle -- e.g. `&[(7.0, 0.15)]` for a straight triangle-to-heptagon
    /// morph, matching this app's own established look.
    pub knob_stages: Vec<(f32, f32)>,
    pub knob_stage_duration_ms: u64,
}
impl Default for ScrollbarStyle {
    fn default() -> Self {
        Self {
            right_inset_px: 14,
            hit_width_px: 28,
            track_top_pct: 26.0,
            track_bottom_pct: 78.0,
            track_weight: 2,
            track_color: Color::stone(700),
            knob_size_px: 20,
            knob_color: Color::orange(400),
            knob_shadow_color: Color::stone(900),
            knob_stages: DEFAULT_KNOB_STAGES.to_vec(),
            knob_stage_duration_ms: 300,
        }
    }
}

/// A vertical, heptagon-knobbed scrollbar -- extracted from `application/src/toc.rs`
/// (the ToC page's own scrollbar), generalized to any [`foliage::View`]-holding entity's
/// scroll state instead of one specific page's.
///
/// `parent` (where this spawns) and `view_target` (whose `ScrollProgress`/`ScrollTo` it
/// reads and drives) are deliberately allowed to differ, and usually must: any child of a
/// `View`-holder gets that view's own scroll offset subtracted from its resolved position
/// on every cascade (`grid/location.rs`'s `resolution.section.position -= view.offset`),
/// so a scrollbar nested *inside* the view it scrolls would scroll itself out of view the
/// moment it was used. Pass a `parent` that is a sibling of (or otherwise outside) the
/// subtree `view_target` scrolls.
///
/// Returns the root entity -- despawn it (or its own parent) to tear the scrollbar down;
/// nothing about it is self-cleaning beyond that.
pub fn scrollbar(
    tree: &mut Tree,
    parent: Entity,
    view_target: Entity,
    style: ScrollbarStyle,
) -> Entity {
    let root = tree.branch(
        parent,
        Leaf::sprout()
            .at(Location::new().xs(
                100.pct()
                    .as_right()
                    .adjust(-style.right_inset_px)
                    .with(style.hit_width_px.px().as_width()),
                style.track_top_pct.pct().as_top().with(
                    (style.track_bottom_pct - style.track_top_pct)
                        .pct()
                        .as_height(),
                ),
            ))
            .elevate(Elevation::up(4))
            .with((
                Grid::new(1.col().gap(0), 1.row().gap(0)),
                InteractionListener::new(),
                InteractionShape::Rectangle,
            )),
    );
    tree.branch(
        root,
        Line::new(style.track_weight)
            .color(style.track_color)
            .at(Location::new().xs(
                50.pct().as_x().with(0.pct().as_y()),
                50.pct().as_x().with(100.pct().as_y()),
            ))
            .elevate(Elevation::up(1))
            .with(InteractionPropagation::pass_through()),
    );
    let knob = tree.branch(
        root,
        Polygon::new()
            .sides(3.0)
            .rounding(0.0)
            .rotation(0.0)
            .color(style.knob_color)
            // a real (if soon-overwritten) `Location`, not left unset -- `shadow_of`
            // below resolves its own box off `knob`'s *live* `Section` immediately, at
            // spawn, so `knob` needs one to already exist rather than being set only by
            // the `ScrollProgress` reaction's first fire further down (which still runs
            // in this same flush, and is what actually places it correctly).
            .at(Location::new().xs(
                50.pct().as_center_x().with(style.knob_size_px.px().as_width()),
                0.pct().as_center_y().with(style.knob_size_px.px().as_height()),
            ))
            .elevate(Elevation::up(3))
            .with((
                Opacity::new(0.0),
                InteractionListener::new(),
                InteractionShape::Circle,
            )),
    );
    // anchored to `knob` -- `shadow_of` derives its box (offset left+down, sized to
    // match) from `knob`'s own *live* resolved section, so it doesn't need re-deriving
    // whenever `knob`'s own placement (the `ScrollProgress` reaction below) moves it.
    let shadow = shadow_of(tree, knob, Elevation::up(2), style.knob_shadow_color);

    let seq = tree.sequence();
    morph_in(
        tree,
        knob,
        seq,
        &style.knob_stages,
        0.0,
        0,
        style.knob_stage_duration_ms,
    );
    morph_in(
        tree,
        shadow,
        seq,
        &style.knob_stages,
        0.0,
        0,
        style.knob_stage_duration_ms,
    );

    let knob_size_px = style.knob_size_px;
    // render: `view_target`'s scroll position -> knob placement (`shadow_of`'s own Anchor
    // to `knob` keeps the shadow following without needing a write here too). Fires once
    // at spawn and again every time the scroll changes, from a drag on this knob, wheeling
    // over the content directly, or a future unrelated `ScrollTo` write -- one door.
    tree.react::<ScrollProgress, _>(
        view_target,
        move |trigger: Trigger<Insert, ScrollProgress>,
              progress: Query<&ScrollProgress>,
              sections: Query<&Section<Logical>>,
              mut tree: Tree| {
            let y = progress.get(trigger.entity).unwrap().y();
            // half the knob's own size, as a percent of `root`'s live height -- without
            // this, mapping `y` straight onto 0%..100% centers the knob exactly on
            // `root`'s own top/bottom edge at the extremes, and `root` (the knob's
            // immediate `Stem` parent) clips its children to its own bounds.
            let bounds = sections.get(root).unwrap();
            let margin_pct =
                (knob_size_px as f32 / 2.0 / bounds.height() * 100.0).clamp(0.0, 50.0);
            let center_y_pct = margin_pct + y * (100.0 - 2.0 * margin_pct);
            tree.write_to(
                knob,
                Location::new().xs(
                    50.pct().as_center_x().with(knob_size_px.px().as_width()),
                    center_y_pct
                        .pct()
                        .as_center_y()
                        .with(knob_size_px.px().as_height()),
                ),
            );
        },
    );

    // input: drag the knob, or tap anywhere on the track to seek there -- both go through
    // the same `ScrollTo` door `extent_check` resolves against `view_target`'s own live
    // `View`/`Section`, so neither can push the knob further than a real drag over the
    // content itself would ever be allowed to scroll.
    tree.subscribe(
        knob,
        move |_: Trigger<Dragged>,
              interaction: Res<CurrentInteraction>,
              sections: Query<&Section<Logical>>,
              mut tree: Tree| {
            let bounds = sections.get(root).unwrap();
            let pct = ((interaction.click().current.top() - bounds.top()) / bounds.height())
                .clamp(0.0, 1.0);
            tree.write_to(view_target, ScrollTo::y(pct));
        },
    );
    tree.on_click(
        root,
        move |_: Trigger<OnClick>,
              interaction: Res<CurrentInteraction>,
              sections: Query<&Section<Logical>>,
              mut tree: Tree| {
            let bounds = sections.get(root).unwrap();
            let pct = ((interaction.click().current.top() - bounds.top()) / bounds.height())
                .clamp(0.0, 1.0);
            tree.write_to(view_target, ScrollTo::y(pct));
        },
    );

    root
}
