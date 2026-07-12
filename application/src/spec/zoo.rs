//! THE ZOO. Button migrated in full-compressed form (diff it mentally against the real
//! 370-line button.rs), Dropdown and Slider in full, Checkbox/RadioGroup/JoinedButtons
//! compressed to their root/build/react outlines — they are variations, not new shapes.

use foliage::*;

// ===========================================================================
// BUTTON, MIGRATED (today: 370 lines; apply_rounding + recompute_colors called by
// hand at construction, then 9 hand-written subscribes re-calling them. Now: one
// tuple-react + two event bridges + one react + two forwards, zero duplication.)
// ===========================================================================

/// Button's OWN config vocabulary — Primary/Secondary are deleted, nothing shared.
/// End-user pokes the whole appearance at once, as the click-handler always wanted:
///     tree.write_to(btn, ButtonStyle { fg, bg, outline, rounding });
#[derive(Component, Clone)]
pub struct ButtonStyle {
    pub fg: Color,
    pub bg: Color,
    pub outline: i32,
    pub rounding: Rounding,
}
/// Interaction state as a component, so engage/disengage restyling goes through the
/// SAME door as config restyling. Two one-line event bridges replace five subscribes.
#[derive(Component, Copy, Clone, Default)]
pub struct Engagement(pub bool);

impl Sprout for ButtonSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Button {},
            self.style,           // ButtonStyle from builder
            TextValue(self.text), // genuinely-identical semantics may share a type;
            IconValue(self.icon), // that is the author's call, never the library's
            self.font_size,
            Engagement(false),
            InteractionListener::new(),
            Grid::new(1.col().gap(4), 1.row().gap(4)),
        )
    }
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        let panel = kids.spawn(Panel::new().at(/* fill */).elevate(Elevation::up(1)).with((
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
        )));
        // no Location: rounding-dependent, set by the style reaction's first fire —
        // the "don't spawn with a placeholder Location" rule survives, without the
        // old hand-called apply_rounding duplicate.
        let icon = kids.spawn(Icon::new(0).elevate(Elevation::up(2)).with((
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
        )));
        let text = kids.spawn(Text::new("").at(/* centered */).elevate(Elevation::up(2)).with((
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
        )));

        // ONE restyle, subsuming recompute_colors + apply_rounding. Appearance depends
        // on two components; observers natively watch tuples — one registration, one
        // body, fires when EITHER is written (and once at build, per react's contract).
        kids.react_any::<(ButtonStyle, Engagement), _>(
            move |t: Trigger<Insert, (ButtonStyle, Engagement)>,
                  styles: Query<&ButtonStyle>,
                  engagement: Query<&Engagement>,
                  mut tree: Tree| {
                let e = t.event_target();
                let s = styles.get(e).unwrap();
                let engaged = engagement.get(e).unwrap().0;
                // fg/bg swap + outline reveal + icon placement + text visibility,
                // verbatim from today's recompute_colors/apply_rounding bodies,
                // targeting panel / icon / text via capture — one place, every trigger.
                let _ = (panel, icon, text, s, engaged, &mut tree);
            },
        );

        // event -> state bridges (the whole remainder of the old five subscribes):
        kids.tree().subscribe(this, move |t: Trigger<Engaged>, mut tree: Tree| {
            tree.write_to(t.event_target(), Engagement(true));
        });
        kids.tree().subscribe(this, move |t: Trigger<Disengaged>, mut tree: Tree| {
            tree.write_to(t.event_target(), Engagement(false));
        });

        // NOT a pure copy — text also recomputes its width Location — so it stays an
        // explicit react. The boundary between forward and react is visible on purpose.
        kids.react::<TextValue, _>(
            move |t: Trigger<Insert, TextValue>, v: Query<&TextValue>, mut tree: Tree| {
                let value = v.get(t.entity).unwrap().clone();
                tree.write_to(text, value); // primitives take their value components
                // + width Location recompute, as today
            },
        );
        // pure copies: root's component -> child, nothing else. Sugar earns its keep.
        kids.forward::<IconValue>(icon);
        kids.forward::<FontSize>(text);
    }
}

// ===========================================================================
// DROPDOWN — dynamic options + the widget the require-gotcha bit last time.
// In this model the gotcha is unrepresentable: everything spawns at
// photosynthesize-time; reactions exist before any poke can land.
// ===========================================================================
// public face:
//   let dd = Dropdown::new().options(["Rust", "Zig", "C"]).at(..).elevate(..).photosynthesize(tree);
//   tree.write_to(dd, Options(fetched_from_server));        // dynamic, any time
//   tree.subscribe(dd, |t: Trigger<SelectionChanged>, ..| t.event().index);

#[derive(Component, Clone, Default)]
pub struct Options(pub Vec<String>);
#[derive(Component, Copy, Clone, Default)]
pub struct Selected(pub Option<usize>);
#[derive(Component, Copy, Clone, Default)]
pub struct Expanded(pub bool);
#[targeted_event]
#[derive(Copy)]
pub struct SelectionChanged {
    pub index: usize,
}

impl Sprout for DropdownSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (
            Dropdown {},
            Options(self.options),
            Selected(None),
            Expanded(false),
            InteractionListener::new(),
            Grid::default(),
        )
    }
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        // static: the closed field
        let field = kids.spawn(Panel::new().rounding(Rounding::Sm).at(/* .. */).elevate(Elevation::up(1)).with((
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
        )));
        let label = kids.spawn(Text::new("").at(/* .. */).elevate(Elevation::up(2)));
        let chevron = kids.spawn(Icon::new(0).at(/* .. */).elevate(Elevation::up(2)));
        let list = kids.spawn(
            Panel::new()
                .at(/* below field, overlaying siblings */)
                .elevate(Elevation::abs(90))
                .with(Grid::new(1.col(), 40.px().gap(2))),
        );
        kids.tree().disable(list);

        kids.tree().on_click(this, move |t: Trigger<OnClick>, ex: Query<&Expanded>, mut tree: Tree| {
            let e = t.event_target();
            tree.write_to(e, Expanded(!ex.get(e).unwrap().0));
        });

        // dynamic structure door: rows follow Options. This author respawns;
        // a pooling author would look exactly like Hand instead.
        let mut rows: Vec<Entity> = Vec::new();
        kids.react::<Options, _>(move |t: Trigger<Insert, Options>, opts: Query<&Options>, mut tree: Tree| {
            let this = t.event_target();
            for r in rows.drain(..) {
                tree.remove(r);
            }
            for (i, opt) in opts.get(this).unwrap().0.iter().enumerate() {
                let row = Children::new(list, &mut tree).spawn(
                    Text::new(opt)
                        .at(Location::new().xs(
                            1.col().as_left().with(1.col().as_right()),
                            (i + 1).row().as_top().with((i + 1).row().as_bottom()),
                        ))
                        .elevate(Elevation::up(1))
                        .with(InteractionListener::new()),
                );
                tree.on_click(row, move |_: Trigger<OnClick>, mut tree: Tree| {
                    tree.write_to(this, (Selected(Some(i)), Expanded(false)));
                    tree.trigger_targets(SelectionChanged::new(i), this);
                });
                rows.push(row);
            }
        });

        kids.react::<Expanded, _>(move |t: Trigger<Insert, Expanded>, ex: Query<&Expanded>, mut tree: Tree| {
            if ex.get(t.entity).unwrap().0 {
                tree.enable(list); // + chevron flip / open animation
            } else {
                tree.disable(list);
            }
        });

        // tuple-react: the label depends on BOTH — a Selected click must relabel, and
        // so must an Options replacement while something is selected (with a single-
        // component react on Selected alone, that second path would go stale).
        kids.react_any::<(Options, Selected), _>(
            move |t: Trigger<Insert, (Options, Selected)>,
                  sel: Query<&Selected>,
                  opts: Query<&Options>,
                  mut tree: Tree| {
                let this = t.event_target();
                let value = sel.get(this).unwrap().0
                    .and_then(|i| opts.get(this).unwrap().0.get(i).cloned())
                    .unwrap_or_else(|| "Select\u{2026}".to_string());
                tree.write_to(label, TextValue(value));
            },
        );
    }
}

// ===========================================================================
// SLIDER — high-frequency narrow updates; drag and programmatic writes share a door
// ===========================================================================
// public face:
//   let s = Slider::new().value(0.7).at(..).elevate(..).photosynthesize(tree);
//   tree.write_to(s, SliderValue(0.25));
//   tree.subscribe(s, |t: Trigger<ValueChanged>, ..| t.event().value);

#[derive(Component, Copy, Clone, Default)]
pub struct SliderValue(pub f32); // 0.0..=1.0
#[targeted_event]
#[derive(Copy)]
pub struct ValueChanged {
    pub value: f32,
}

impl Sprout for SliderSprout {
    fn seed(&mut self) -> &mut LeafSprout {
        &mut self.leaf
    }
    fn root(self) -> impl Bundle {
        (Slider {}, SliderValue(self.value), Grid::default())
    }
    fn build<T: EcsExtension>(this: Entity, kids: &mut Children<T>) {
        let track = kids.spawn(Line::new(4).color(Color::gray(700)).at(/* full width, mid */).elevate(Elevation::up(1)));
        let fill = kids.spawn(Line::new(4).color(Color::green(300)).at(/* zero-width, mid */).elevate(Elevation::up(2)));
        // thumb rides the fill's end via Anchor — music_player's scrubber, now reusable
        let thumb = kids.spawn(
            Panel::new()
                .rounding(Rounding::Full)
                .at(/* anchored to fill end */)
                .elevate(Elevation::up(3))
                .with((Anchor::new(fill), InteractionListener::new())),
        );

        // input: drag -> value. Touches ONE component; drawing happens below.
        kids.tree().subscribe(
            thumb,
            move |_: Trigger<Dragged>,
                  interaction: Res<CurrentInteraction>, // GAP-2: must be pub for authors
                  sections: Query<&Section<Logical>>,
                  mut tree: Tree| {
                let t = sections.get(track).unwrap();
                let pct = ((interaction.click.current.left() - t.left()) / t.width()).clamp(0.0, 1.0);
                tree.write_to(this, SliderValue(pct));
            },
        );

        // render: value -> geometry. Identical for drag writes and programmatic writes.
        kids.react::<SliderValue, _>(
            move |t: Trigger<Insert, SliderValue>, vals: Query<&SliderValue>, mut tree: Tree| {
                let v = vals.get(t.entity).unwrap().0;
                tree.write_to(
                    fill,
                    Location::new().xs(
                        0.pct().as_x().with(50.pct().as_y()),
                        (v * 100.0).pct().as_x().with(50.pct().as_y()),
                    ),
                );
                tree.trigger_targets(ValueChanged::new(v), t.event_target());
            },
        );
    }
}

// ===========================================================================
// CHECKBOX (compressed) — the Button-shaped trivial case
// ===========================================================================
// root:  (Checkbox {}, Checked(bool), CheckStyle { .. }, InteractionListener)
// build: box panel + check icon (static)
//   on_click(this):              read Checked -> write_to(this, Checked(!v))
//   react::<(Checked, CheckStyle)>: icon visibility + fill + colors, one body;
//                                emit Toggled::new(checked)
// Nothing else. Small widgets stay small.

// ===========================================================================
// RADIOGROUP (compressed) — config-dependent structure, the rule's consequence
// ===========================================================================
// root:  (RadioGroup {}, Choices(Vec<String>), Choice(Option<usize>), Grid rows)
// build: NO static children — row count depends on Choices, so structure goes
//        through the reaction door:
//   react::<Choices>: (re)spawn one row per choice (dot leaf + label text),
//                     on_click(row i) -> write_to(this, Choice(Some(i)))
//   react::<Choice>:  restyle dots; emit ChoiceChanged::new(index)
// Identical shape to Dropdown's options and KeyedHand. One pattern, learned once.

// ===========================================================================
// JOINEDBUTTONS / segmented control (compressed) — widgets authored FROM widgets
// ===========================================================================
// root:  (Joined {}, Segments(Vec<String>), Active(usize), Grid cols)
// build: none static
//   react::<Segments>: (re)spawn one Button::new().text(s) per segment — nested
//                      widgets configured through their PUBLIC components, exactly
//                      as an end-user would; on_click(button i) -> write Active(i).
//                      Position-dependent config is just code in this reaction:
//                        rounding: match i { 0 => left edge, last => right edge,
//                                            _ => none }
//                      (per-edge Rounding on the Panel primitive is GAP-4 — the
//                      authoring model already expresses it; the primitive can't
//                      draw it yet.)
//   react::<Active>:   write_to(button_j, ButtonStyle{..}) for all j — restyling
//                      children by poking their public face; emit ActiveChanged::new(i)
