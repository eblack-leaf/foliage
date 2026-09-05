use core::ops::Range;

use crate::aspen::{Motion, Sequence, Timing, Tween};
use crate::coordinate::Area;
use crate::elevation::Elevation;
use crate::image::Plate;
use crate::interaction::focus::Intent;
use crate::leaf::{Growth, Leaf};
use crate::op::Op;
use crate::palette::{Fill, Scheme};
use crate::placement::grid::Grid;
use crate::placement::location::Location;
use crate::placement::point::Point;
use crate::polygon::Shape;
use crate::rounding::Corners;
use crate::seed::Seed;
use crate::text::Tints;
use crate::view::ScrollTo;

/// What an op sink has to be able to do: take an op, and hand out the names an op may need.
///
/// Naming an element hands back its place in allocation order along with the name, because that
/// order is fixed where the name is asked for and not where the element is grown.
pub(crate) trait Queues {
    fn queue(&mut self, op: Op);
    fn allocate(&self) -> (Leaf, Growth);
    fn name(&self) -> Tween;
    fn group(&self) -> Sequence;
    /// A name for a picture whose pixels are on their way. Taken here rather than at the drain so
    /// that an element can be grown against it in the frame it was asked for.
    fn picture(&mut self) -> Plate;
}

/// Everything an app can ask the engine to do.
///
/// A change reads identically wherever it is issued. Sealed: it can be called, never implemented.
#[allow(private_bounds)]
pub trait Grow: Queues {
    /// Grows a top-level element and hands back the [`Leaf`] naming it. Usable immediately,
    /// including as a trunk in the same frame.
    #[track_caller]
    fn plant(&mut self, seed: impl Seed) -> Leaf {
        let (leaf, growth) = self.allocate();
        self.queue(Op::Plant {
            leaf,
            growth,
            bud: seed.bud(core::panic::Location::caller()),
        });
        leaf
    }

    /// Grows an element off `under`.
    #[track_caller]
    fn branch(&mut self, under: Leaf, seed: impl Seed) -> Leaf {
        let (leaf, growth) = self.allocate();
        self.queue(Op::Branch {
            leaf,
            growth,
            under,
            bud: seed.bud(core::panic::Location::caller()),
        });
        leaf
    }

    /// Takes an element and everything beneath it down. Each one is reported as
    /// [`withered`](crate::Pollen::withered).
    fn prune(&mut self, leaf: Leaf) {
        self.queue(Op::Prune(leaf));
    }

    /// Moves an element, replacing its whole placement.
    ///
    /// A placement is one value rather than a set of edges, so there is no half-written state
    /// between two of these and no question of which edge a later write meant.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element is placed by its
    /// ends rather than by a box -- which is [`between`](Grow::between).
    fn at(&mut self, leaf: Leaf, location: Location) {
        self.queue(Op::Place { leaf, location });
    }

    /// Moves both ends of a stroke.
    ///
    /// The point-mode counterpart to [`at`](Grow::at), and separate for the same reason the two
    /// declarations are: an element has a box or it has ends, and a verb that wrote either would be
    /// able to write the one the element does not have.
    ///
    /// Both ends together, because a stroke is one thing: a verb per end would leave a frame in
    /// which the line is half moved.
    ///
    /// Dropped if the element is placed by a box.
    fn between(&mut self, leaf: Leaf, from: Point, to: Point) {
        self.queue(Op::Trace { leaf, from, to });
    }

    /// Redivides an element's box for the elements grown under it.
    fn grid(&mut self, leaf: Leaf, grid: Grid) {
        self.queue(Op::Divide { leaf, grid });
    }

    /// Points an element's placement at the one other element it may read.
    ///
    /// Replaces any anchor it already had.
    ///
    /// # Panics
    ///
    /// If the anchor would close a cycle, on the same terms as
    /// [`anchored`](crate::Place::anchored).
    #[track_caller]
    fn anchor(&mut self, leaf: Leaf, to: Leaf) {
        self.queue(Op::Anchor {
            leaf,
            to,
            at: core::panic::Location::caller(),
        });
    }

    /// Raises or lowers an element, and everything grown under it with it.
    ///
    /// Elevation accumulates down the tree, so this moves a whole subtree by one write and nothing
    /// inside it is touched.
    fn elevate(&mut self, leaf: Leaf, elevation: Elevation) {
        self.queue(Op::Elevate { leaf, elevation });
    }

    /// Refills an element, with a [`Palette`](crate::Palette) role or with a [`Color`](crate::Color) stated
    /// outright.
    ///
    /// A role is the ordinary answer, and a literal is an element saying it is not part of the
    /// scheme: a [`repaint`](Grow::repaint) moves the first and not the second.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element draws nothing.
    fn color(&mut self, leaf: Leaf, fill: impl Into<Fill>) {
        self.queue(Op::Recolor {
            leaf,
            fill: fill.into(),
        });
    }

    /// Rewrites what a run of glyphs says.
    ///
    /// The measure follows in the same frame: R1 shapes what is written here and R2m wraps it, both
    /// of them before anything reads a box, so an element sized to its own content is the right size
    /// on the frame the string changed rather than the frame after.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element is not a
    /// [`Text`](crate::Text).
    fn text(&mut self, leaf: Leaf, value: impl Into<String>) {
        self.queue(Op::Letter {
            leaf,
            value: value.into(),
        });
    }

    /// Fills parts of a run differently from the rest of it, over its own index space.
    ///
    /// Replaces every tint on the run rather than adding one, for the reason a placement is one
    /// value: there is no half-written state between two of these, and no question of which range a
    /// later write meant. Handing it nothing clears them.
    ///
    /// ```no_run
    /// # use foliage::{Grow, Grove, Leaf, Palette};
    /// # fn f(grove: &mut Grove, run: Leaf) {
    /// grove.tint(run, [(0..2, Palette::Accent), (7..12, Palette::Muted)]);
    /// # }
    /// ```
    ///
    /// [`untint`](Grow::untint) is how they come off, rather than handing this an empty set: the
    /// fill type of a set with nothing in it cannot be inferred, and a verb that has to be told the
    /// type of what it is not writing is a worse surface than a second verb.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element is not a
    /// [`Text`](crate::Text).
    fn tint<F: Into<Fill>>(
        &mut self,
        leaf: Leaf,
        tints: impl IntoIterator<Item = (Range<usize>, F)>,
    ) {
        self.queue(Op::Tint {
            leaf,
            tints: Tints(
                tints
                    .into_iter()
                    .map(|(range, fill)| (range, fill.into()))
                    .collect(),
            ),
        });
    }

    /// Takes every tint off a run, leaving the whole of it in the run's own
    /// [`color`](Grow::color).
    ///
    /// Dropped, like any op naming something it does not apply to, if the element is not a
    /// [`Text`](crate::Text).
    fn untint(&mut self, leaf: Leaf) {
        self.queue(Op::Tint {
            leaf,
            tints: Tints::default(),
        });
    }

    /// Selects a span of a [`TextInput`](crate::TextInput)'s value, in characters.
    ///
    /// The caret goes to the range's end and the selection is anchored at its start, so an empty
    /// range places a caret and nothing else -- which is what makes this one verb rather than two
    /// that could disagree about where the caret is.
    ///
    /// Both ends are held inside the value, so a range past it selects to the end.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element is not a field.
    fn select(&mut self, leaf: Leaf, range: Range<usize>) {
        self.queue(Op::Select { leaf, range });
    }

    /// Rounds an element's corners, per corner or all at once.
    ///
    /// Dropped, like any op naming something it does not apply to, unless the element is a
    /// rectangle -- a [`Panel`](crate::Panel) or an [`Image`](crate::Image). A
    /// [`Polygon`](crate::Polygon)'s corners are its own, and are moved with
    /// [`reshape`](Grow::reshape).
    fn round(&mut self, leaf: Leaf, rounding: impl Into<Corners>) {
        self.queue(Op::Round {
            leaf,
            rounding: rounding.into(),
        });
    }

    /// Reshapes a regular polygon: how many sides, how round its corners, how far it is turned.
    ///
    /// One value rather than three verbs, because it is one thought and because
    /// [`Motion::Polygon`](crate::Motion::Polygon) moves it as one.
    ///
    /// Dropped if the element is not a [`Polygon`](crate::Polygon).
    fn reshape(&mut self, leaf: Leaf, shape: Shape) {
        self.queue(Op::Reshape { leaf, shape });
    }

    /// Registers a picture and hands back the name elements draw it by.
    ///
    /// `pixels` is RGBA, one byte per channel, row-major, `size` texels across -- pixels the app
    /// made rather than a file it has. An encoded picture, and one read from a path or a URL, is
    /// [`Grove::image`](crate::Grove::image), which decodes and takes its size from what it decoded.
    ///
    /// Usable at any frame. A name taken here is valid immediately -- elements can be grown against
    /// it in the same frame -- and writing the same name again replaces what it holds, so a picture
    /// re-rendered at a higher resolution reaches every element drawing it with one write.
    fn pixels(&mut self, pixels: impl Into<Vec<u8>>, size: Area) -> Plate {
        let plate = self.plate();
        self.load(plate, pixels, size);
        plate
    }

    /// Names a picture whose pixels have not arrived.
    ///
    /// The two halves of [`pixels`](Grow::pixels), for when they happen at different times. A name is
    /// valid the moment it is handed out, so elements can be grown against it now and
    /// [`load`](Grow::load)ed when a fetch or a decode finishes -- an element drawing a plate with
    /// nothing behind it occupies its box, draws nothing, and appears on the frame its pixels do.
    ///
    /// That is what keeps "is it loaded yet" out of an app's state. There is no readback for it and
    /// deliberately so: the answer only ever changes what is on screen, and the engine already
    /// changes that.
    fn plate(&mut self) -> Plate {
        self.picture()
    }

    /// Fills a picture's name with pixels.
    ///
    /// `pixels` is RGBA, one byte per channel, row-major, `size` texels across. Writing a name that
    /// already holds a picture replaces it, so a re-fetch at a higher resolution reaches every
    /// element drawing it without any of them being named.
    ///
    /// # Panics
    ///
    /// If `pixels` is smaller than `size` texels of RGBA.
    fn load(&mut self, plate: Plate, pixels: impl Into<Vec<u8>>, size: Area) {
        self.queue(Op::Load {
            plate,
            pixels: pixels.into(),
            size,
        });
    }

    /// Moves a scrolling region to a stated place.
    ///
    /// The destination is answered against the extent of the frame it lands in, so scrolling to the
    /// end of a list that grew in the same frame lands at the end of the list rather than where the
    /// list used to stop. It is clamped to what the region can reach, and reading the offset back
    /// afterwards returns the pixels it settled at.
    ///
    /// ```no_run
    /// # use foliage::{Grove, Grow, Leaf, ScrollTo};
    /// # fn f(grove: &mut Grove, column: Leaf, section: Leaf) {
    /// grove.scroll(column, ScrollTo::px(240.0));
    /// grove.scroll(column, ScrollTo::show(section));
    /// # }
    /// ```
    ///
    /// A direct write, so it cancels a [`Motion::Scroll`](crate::Motion::Scroll) still moving the
    /// region, and it ends a coast the reader's last drag left running.
    ///
    /// Dropped, like any op naming something it does not apply to, if the element does not scroll,
    /// if the destination leaves no axis to move, or if
    /// [`ScrollTo::show`](ScrollTo::show) names an element not grown under it.
    fn scroll(&mut self, leaf: Leaf, to: ScrollTo) {
        self.queue(Op::Scroll { leaf, to });
    }

    /// Makes an element inert without taking it out of the picture.
    ///
    /// It still draws -- a greyed control is still a control -- and it still occupies the box
    /// stack, so it **swallows**: a press on it reaches neither the element itself nor anything
    /// behind it, and a drag over it scrolls nothing. That is the whole difference between disabled
    /// and decoration, and it is what makes disabling a page enough on its own when a drawer opens
    /// over it: the page goes inert without a scrim to arrange.
    ///
    /// Cascades to everything grown under it, as a product recomputed every frame rather than a
    /// write pushed down -- so a child grown under a disabled element is disabled on its first
    /// frame. Focus leaves the subtree if it was in it.
    fn disable(&mut self, leaf: Leaf) {
        self.queue(Op::Disable {
            leaf,
            disabled: true,
        });
    }

    /// Undoes [`disable`](Grow::disable) on this element.
    ///
    /// Symmetric: an element inside the subtree that was disabled in its own right stays disabled,
    /// because what is recomputed is the product over the whole ancestry and not one inherited bit
    /// that was overwritten on the way down.
    fn enable(&mut self, leaf: Leaf) {
        self.queue(Op::Disable {
            leaf,
            disabled: false,
        });
    }

    /// Whether an element is drawn at all, as [`visible`](crate::Place::visible) states it at
    /// spawn.
    fn visible(&mut self, leaf: Leaf, visible: bool) {
        self.queue(Op::Reveal { leaf, visible });
    }

    /// How opaque an element is, as [`opacity`](crate::Place::opacity) states it at spawn.
    fn opacity(&mut self, leaf: Leaf, opacity: f32) {
        self.queue(Op::Fade { leaf, opacity });
    }

    /// Moves a property of an element over time.
    ///
    /// The target is written to the element at once and the tween carries what it left, so the
    /// element declares where it is going from the moment it is told to go there. Nothing has to be
    /// undone when it arrives: the blend at the end is the plain reading of the declaration.
    ///
    /// A second `animate` on a property already moving replaces it, starting from where the element
    /// currently is rather than from where the first one began. A **direct write** to that property
    /// -- [`at`](Grow::at), [`color`](Grow::color), [`opacity`](Grow::opacity) -- cancels it, and
    /// the element is at what was written. So a property that is animated somewhere does not have
    /// to be animated everywhere.
    ///
    /// ```no_run
    /// # use foliage::{Color, Ease, Grow, Grove, Motion, Palette, Timing};
    /// # fn f(grove: &mut Grove, leaf: foliage::Leaf) {
    /// grove.animate(leaf, Motion::Palette(Palette::Accent), Timing::ms(180));
    /// grove.animate(leaf, Motion::Color(Color::rgb(1.0, 0.4, 0.2)), Timing::ms(180));
    /// grove.animate(leaf, Motion::Opacity(0.0), Timing::ms(240).ease(Ease::Accelerate));
    /// # }
    /// ```
    ///
    /// # The name it hands back
    ///
    /// This motion's own, for the app that has to tell one of an element's motions from another:
    /// [`Pollen::tween`] is how far it has come and [`Pollen::finished`] is the frame it arrived,
    /// against [`Pollen::landed`], which is the element settling and says nothing about which of its
    /// properties did. [`stop`](Grow::stop) ends it, and [`Timing::within`](crate::Timing::within)
    /// counts it into a group as ever.
    ///
    /// ```no_run
    /// # use foliage::{Grow, Grove, Motion, Pollen, Timing};
    /// # fn f(grove: &mut Grove, pollen: &Pollen, card: foliage::Leaf) {
    /// let fade = grove.animate(card, Motion::Opacity(0.0), Timing::ms(240));
    /// if pollen.finished(fade) {
    ///     grove.prune(card);
    /// }
    /// # }
    /// ```
    ///
    /// A name reports for as long as the motion it named is the one running, and goes quiet
    /// otherwise: a motion replaced by a second `animate` on the same property, cancelled by a
    /// direct write, stopped, or taken down with its element reports nothing further. So does one
    /// that never ran -- dropped, like any op naming something it does not apply to, because the
    /// element cannot carry the motion, as a fill cannot be moved on something that draws nothing.
    ///
    /// [`Pollen::tween`]: crate::Pollen::tween
    /// [`Pollen::finished`]: crate::Pollen::finished
    /// [`Pollen::landed`]: crate::Pollen::landed
    fn animate(&mut self, leaf: Leaf, motion: Motion, timing: Timing) -> Tween {
        let tween = self.name();
        self.queue(Op::Animate {
            leaf,
            motion,
            timing,
            tween,
        });
        tween
    }

    /// Runs a number from `from` to `to`, reported each frame as [`Pollen::tween`] and written
    /// nowhere.
    ///
    /// The engine's clock and easing, made available to a value it has no concept of. This is the
    /// answer for everything [`Motion`] deliberately leaves out and for anything an app invents:
    /// [`Motion`] is closed because the *engine's* obligations should be, not because an app's are.
    ///
    /// The frame it ends reports its end value and [`Pollen::finished`] together.
    ///
    /// [`Pollen::tween`]: crate::Pollen::tween
    /// [`Pollen::finished`]: crate::Pollen::finished
    fn tween(&mut self, from: f32, to: f32, timing: Timing) -> Tween {
        let tween = self.name();
        self.queue(Op::Channel {
            tween,
            from,
            to,
            timing,
        });
        tween
    }

    /// A [`tween`](Grow::tween) whose value is not read: what is wanted is the report that the time
    /// is up.
    ///
    /// A timer of zero fires no earlier than the next frame -- queued at step 3, applied at 4,
    /// advanced at 5, and reported at step 3 of the frame after. Honest rather than special-cased.
    fn timer(&mut self, timing: Timing) -> Tween {
        self.tween(0.0, 1.0, timing)
    }

    /// Names a group of tweens, so their *last* ending is reported as well as each of their own.
    ///
    /// A handle and nothing else. Anything that runs on the clock joins one with
    /// [`Timing::within`](crate::Timing::within), from any callsite and at any frame, which is the
    /// whole point: a sequence exists to time together things that have no reason to be written
    /// together, and one that had to be declared in a single call would not be able to.
    ///
    /// ```no_run
    /// # use foliage::{Ease, Grow, Grove, Motion, Palette, Timing};
    /// # fn f(grove: &mut Grove, first: foliage::Leaf, second: foliage::Leaf) {
    /// let intro = grove.sequence();
    /// grove.animate(first, Motion::Opacity(1.0), Timing::ms(200).within(intro));
    /// grove.animate(
    ///     second,
    ///     Motion::Palette(Palette::Accent),
    ///     Timing::ms(200).after(80).ease(Ease::Decelerate).within(intro),
    /// );
    /// # }
    /// ```
    ///
    /// The offsets stay on each tween, where [`after`](crate::Timing::after) already puts them: a
    /// group says *when it is over*, not when its members start, so there is one place a delay is
    /// stated rather than two that can disagree.
    ///
    /// Reported as [`sequence_finished`](crate::Pollen::sequence_finished) the frame nothing is
    /// running under it any more. A name that has emptied can be filled again.
    fn sequence(&mut self) -> Sequence {
        self.group()
    }

    /// Ends a tween before it has run out. **Nothing is reported**, so a chain waiting on it does
    /// not run.
    ///
    /// A motion is left on its target: that is what the element was told to be from the moment the
    /// motion started, and the duration was only how long it was to take getting there, so giving up
    /// the duration leaves it where it was meant to end. To end it somewhere *else* is to write that
    /// property, which cancels the motion and puts the element at what was written -- the engine
    /// never writes back a value from part way along, because a blend part way along is a
    /// synthesized intermediate the app never declared.
    ///
    /// Nothing is reported because what stopped it already knows: this is the app's own act, and an
    /// app's own writes are never reported back to it. Stopping the rest of a chain is calling this
    /// on their names at the same callsite.
    ///
    /// A group counting it is told, because a group is over when nothing is running under it however
    /// each member ended -- so stopping the last member ends the group. It has no reach beyond the
    /// tween named: a [`Sequence`] is a name for when a set is quiet, not something that owns what
    /// joined it, and stopping one member is nothing to the others.
    ///
    /// Dropped silently if it has already ended.
    fn stop(&mut self, tween: Tween) {
        self.queue(Op::Stop {
            tween,
            reported: false,
        });
    }

    /// Ends a tween before it has run out, **as an arrival**: a chain waiting on it runs this frame.
    ///
    /// [`stop`](Grow::stop) and nothing else, except that the ending is reported. A motion reports
    /// [`landed`](crate::Pollen::landed) and [`finished`](crate::Pollen::finished) as though it had
    /// run its course; a channel reports its end value and its finish together, as it would have on
    /// its own last frame; a [`timer`](Grow::timer) fires.
    ///
    /// What an interruption wants whenever the thing waiting on the motion is the point of it.
    /// Dismissing a card mid-fade should still take the card down, and it should do so through the
    /// one path that takes it down:
    ///
    /// ```no_run
    /// # use foliage::{Grow, Grove, Motion, Pollen, Timing};
    /// # fn f(grove: &mut Grove, pollen: &Pollen, card: foliage::Leaf, fade: foliage::Tween) {
    /// if pollen.root_keys().is_empty() {
    ///     grove.finish(fade);
    /// }
    /// if pollen.finished(fade) {
    ///     grove.prune(card);
    /// }
    /// # }
    /// ```
    ///
    /// Dropped silently if it has already ended.
    fn finish(&mut self, tween: Tween) {
        self.queue(Op::Stop {
            tween,
            reported: true,
        });
    }

    /// Moves focus to an element.
    ///
    /// The ordinary way focus moves. It is not a byproduct of pressing anything: a press moves
    /// focus nowhere, so an app that wants a field focused when it is tapped writes that from
    /// [`clicked`](crate::Pollen::clicked), and the engine never guesses.
    ///
    /// Dropped if the element cannot take focus -- it is not
    /// [`interactive`](crate::Place::interactive), or it is hidden or disabled. Focus stays where
    /// it was rather than moving somewhere the app did not name.
    fn focus(&mut self, leaf: Leaf) {
        self.queue(Op::Focus(Intent::To(leaf)));
    }

    /// Takes focus off whatever holds it, leaving nothing focused.
    fn unfocus(&mut self) {
        self.queue(Op::Focus(Intent::Away));
    }

    /// Moves focus to the next element in reading order, wrapping within the scope focus is in.
    ///
    /// With nothing focused, this takes the first.
    fn focus_next(&mut self) {
        self.queue(Op::Focus(Intent::Next));
    }

    /// Moves focus to the previous element in reading order, wrapping within the scope focus is in.
    ///
    /// With nothing focused, this takes the last.
    fn focus_previous(&mut self) {
        self.queue(Op::Focus(Intent::Previous));
    }

    /// Puts text on the system clipboard.
    ///
    /// The app's own copy. A [`TextInput`](crate::TextInput) already answers `Ctrl+C` and `Ctrl+X`
    /// for itself, so this is for everything that is not a field -- a code block, a share link, a
    /// row of a table.
    fn copy(&mut self, text: impl Into<String>) {
        self.queue(Op::Copy(text.into()));
    }

    /// Asks what the system clipboard holds, reported as [`pasted`](crate::Pollen::pasted).
    ///
    /// Never in the frame that asked: a clipboard is a promise in a browser and a round trip to
    /// whichever program owns the selection on a desktop, so what it holds arrives the way bytes
    /// from a path do and is drained in the frame it arrives in.
    ///
    /// A field answers `Ctrl+V` for itself and does not go through this. What comes back here is
    /// the app's, whatever holds focus.
    ///
    /// An empty answer is what an app gets for an empty clipboard and for one the host would not
    /// let it read -- a browser outside a user gesture, a desktop with no display server. There is
    /// one thing to do about either, so they are one outcome and the reason is traced.
    fn paste(&mut self) {
        self.queue(Op::Paste { into: None });
    }

    /// Goes to `url`.
    ///
    /// On the web this **replaces the page**, and deliberately: a new tab is blocked outside a user
    /// gesture, and a frame is not one. Off the web the desktop is asked to open the URL, which is
    /// what going somewhere means where there is no page to replace.
    fn navigate(&mut self, url: impl Into<String>) {
        self.queue(Op::Navigate(url.into()));
    }

    /// Asks the host to save what is at `url`.
    ///
    /// The web's, and only the web's -- a browser is what turns a URL into a file in someone's
    /// downloads. Off the web it is traced and nothing else, so an app naming what it offers names
    /// it once.
    fn download(&mut self, url: impl Into<String>) {
        self.queue(Op::Download(url.into()));
    }

    /// States what every [`Palette`](crate::Palette) tone resolves to, for the whole tree.
    ///
    /// The one write that names no element, because a tone belongs to the scheme and not to any of
    /// the elements declaring it. Everything painted in a tone whose color changed is re-extracted
    /// and nothing else is, which is what makes a theme one op rather than a walk.
    fn repaint(&mut self, scheme: Scheme) {
        self.queue(Op::Repaint(scheme));
    }
}

impl<T: Queues> Grow for T {}
