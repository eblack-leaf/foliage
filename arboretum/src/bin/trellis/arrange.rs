//! Arranging the sheet: a force-directed layout, worked out in plain arithmetic.
//!
//! Every card pushes every other away, every wire pulls its two ends together, and each kind of
//! thing leans toward its own band of the sheet -- the cast on the left, then the meetings, the
//! specs, and the milestones -- so a plan arranged reads left to right the way it is worked. The
//! forces settle the order; then each band is ruled into a straight column in that order, a few
//! sweeps draw every card toward the middle of what it is wired to (which uncrosses the wires),
//! each note is set beside what it annotates, whatever still overlaps is pushed apart, everything
//! is kept inside the drawing and off the blocks printed on it, and every corner is snapped to the
//! sheet's 8px.
//!
//! Nothing random: the layout starts from where everything already is, so the same board always
//! arranges the same way.

use crate::board::Kind;

/// One card to place.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Body {
    pub kind: Kind,
    /// Its top left corner.
    pub at: (f32, f32),
    pub size: (f32, f32),
}

/// Where cards may stand: a box, less the boxes printed on it.
#[derive(Clone, Debug, PartialEq)]
pub struct Bounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    /// `(left, top, right, bottom)` boxes nothing may stand on: the title block, the revisions.
    pub keep_out: Vec<(f32, f32, f32, f32)>,
}

/// How far apart two cards stand at the least.
pub const GAP: f32 = 20.0;
/// The grid everything snaps to.
pub const SNAP: f32 = 8.0;
const STEPS: usize = 260;
const UNTANGLE: usize = 600;
const SWEEPS: usize = 8;

/// Where each of `bodies` stands once arranged, as top left corners, in the same order. `edges`
/// name bodies by where they stand in `bodies`.
pub fn arrange(bodies: &[Body], edges: &[(usize, usize)], bounds: &Bounds) -> Vec<(f32, f32)> {
    let n = bodies.len();
    if n == 0 {
        return Vec::new();
    }
    let width = (bounds.right - bounds.left).max(1.0);
    let height = (bounds.bottom - bounds.top).max(1.0);
    // The distance a wire likes to be.
    let k = (width * height / n as f32).sqrt() * 0.62;
    let mut centre: Vec<(f32, f32)> = bodies
        .iter()
        .enumerate()
        .map(|(i, b)| {
            // Two cards on one spot are drawn a little apart, by where they stand in the list.
            let nudge = (i as f32 * 0.618).fract() - 0.5;
            (b.at.0 + b.size.0 / 2.0 + nudge, b.at.1 + b.size.1 / 2.0 + nudge * 1.3)
        })
        .collect();
    // Where each kind's column stands: at its share of the width, or clear of the widest card in
    // the column before it, whichever is further right.
    let mut columns = [0.0f32; 4];
    let mut reach = bounds.left;
    for (c, (kind, share)) in [(Kind::Person, 0.07), (Kind::Meeting, 0.32), (Kind::Spec, 0.58), (Kind::Milestone, 0.80)]
        .into_iter()
        .enumerate()
    {
        columns[c] = (bounds.left + width * share).max(reach);
        let widest = bodies.iter().filter(|b| b.kind == kind).map(|b| b.size.0).fold(0.0, f32::max);
        reach = columns[c] + widest + GAP * 2.0;
    }
    let band = |kind: Kind| -> Option<f32> {
        match kind {
            Kind::Person => Some(columns[0]),
            Kind::Meeting => Some(columns[1]),
            Kind::Spec => Some(columns[2]),
            Kind::Milestone => Some(columns[3]),
            Kind::Note => None,
        }
    };
    let mut heat = k * 0.4;
    for _ in 0..STEPS {
        let mut push = vec![(0.0f32, 0.0f32); n];
        for i in 0..n {
            for j in i + 1..n {
                let (dx, dy) = (centre[i].0 - centre[j].0, centre[i].1 - centre[j].1);
                // Measured between the cards' edges rather than their middles, so a wide card is
                // given the room it takes.
                let reach = (bodies[i].size.0 + bodies[j].size.0) * 0.32 + (bodies[i].size.1 + bodies[j].size.1) * 0.3;
                let d = (dx.hypot(dy) - reach * 0.5).max(8.0);
                let f = k * k / d / dx.hypot(dy).max(0.01);
                push[i].0 += dx * f;
                push[i].1 += dy * f;
                push[j].0 -= dx * f;
                push[j].1 -= dy * f;
            }
        }
        for &(a, b) in edges {
            if a >= n || b >= n || a == b {
                continue;
            }
            let (dx, dy) = (centre[a].0 - centre[b].0, centre[a].1 - centre[b].1);
            let d = dx.hypot(dy).max(0.01);
            let f = d / k;
            push[a].0 -= dx * f;
            push[a].1 -= dy * f;
            push[b].0 += dx * f;
            push[b].1 += dy * f;
        }
        for i in 0..n {
            if let Some(x) = band(bodies[i].kind) {
                push[i].0 += (x + bodies[i].size.0 / 2.0 - centre[i].0) * 4.0;
            }
            // A little toward the middle, so nothing drifts to an edge for want of a reason.
            push[i].1 += (bounds.top + height / 2.0 - centre[i].1) * 0.15;
            let length = push[i].0.hypot(push[i].1);
            if length > 0.0 {
                let step = length.min(heat);
                centre[i].0 += push[i].0 / length * step;
                centre[i].1 += push[i].1 / length * step;
            }
            centre[i] = held(centre[i], bodies[i].size, bounds);
        }
        heat = (heat * 0.985).max(1.5);
    }
    // The forces settle the order; the columns are then ruled straight. Each banded kind stands
    // in its own column, in the order the forces left it, packed down it a gap apart and lifted
    // back inside the drawing if it ran past the foot.
    // Packs one column: each card as near as it can stand to where it wants to, a gap under the
    // one above, and clear of anything else that reaches across the column -- a wide card's
    // markup from the column before, a block printed on the sheet. Packed from the top, and if
    // that runs past the foot, again from the foot up.
    let rule = |centre: &mut Vec<(f32, f32)>, kind: Kind, wants: &[f32]| {
        let Some(x) = band(kind) else {
            return;
        };
        let mut column: Vec<usize> = (0..n).filter(|&i| bodies[i].kind == kind).collect();
        column.sort_by(|&a, &b| wants[a].total_cmp(&wants[b]).then(a.cmp(&b)));
        let mut obstacles: Vec<(f32, f32, f32, f32)> = bounds.keep_out.clone();
        obstacles.extend((0..n).filter(|&j| bodies[j].kind != kind && band(bodies[j].kind).is_some()).map(|j| {
            let (w, h) = bodies[j].size;
            (centre[j].0 - w / 2.0, centre[j].1 - h / 2.0, centre[j].0 + w / 2.0, centre[j].1 + h / 2.0)
        }));
        // Whatever stands in the way of a card `w` across and `h` down with its top at `top`.
        let blocking = |w: f32, h: f32, top: f32| {
            obstacles
                .iter()
                .copied()
                .find(|&(l, t, r, b)| x < r + GAP && x + w + GAP > l && top < b + GAP && top + h + GAP > t)
        };
        let mut floor = bounds.top;
        for &i in &column {
            let (w, h) = bodies[i].size;
            let mut top = (wants[i] - h / 2.0).max(floor);
            for _ in 0..n + 4 {
                match blocking(w, h, top) {
                    Some((_, _, _, b)) => top = b + GAP,
                    None => break,
                }
            }
            centre[i] = (x + w / 2.0, top + h / 2.0);
            floor = top + h + GAP;
        }
        if floor - GAP > bounds.bottom {
            let mut ceiling = bounds.bottom;
            for &i in column.iter().rev() {
                let (w, h) = bodies[i].size;
                let mut top = (centre[i].1 - h / 2.0).min(ceiling - h);
                for _ in 0..n + 4 {
                    match blocking(w, h, top) {
                        Some((_, t, _, _)) => top = t - GAP - h,
                        None => break,
                    }
                }
                centre[i] = (x + w / 2.0, top + h / 2.0);
                ceiling = top - GAP;
            }
        }
        for &i in &column {
            centre[i] = held(centre[i], bodies[i].size, bounds);
        }
    };
    const RULED: [Kind; 4] = [Kind::Person, Kind::Meeting, Kind::Spec, Kind::Milestone];
    let here: Vec<f32> = centre.iter().map(|c| c.1).collect();
    for kind in RULED {
        rule(&mut centre, kind, &here);
    }
    // Then a few sweeps across the columns and back, each card drawn toward the middle of what
    // it is wired to: the classic way to uncross a layered drawing.
    let mut wired: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in edges {
        if a < n && b < n && a != b && band(bodies[a].kind).is_some() && band(bodies[b].kind).is_some() {
            wired[a].push(b);
            wired[b].push(a);
        }
    }
    for sweep in 0..SWEEPS {
        let order: Vec<Kind> = match sweep % 2 {
            0 => RULED[1..].to_vec(),
            _ => RULED[..3].iter().rev().copied().collect(),
        };
        for kind in order {
            let wants: Vec<f32> = (0..n)
                .map(|i| match wired[i].len() {
                    0 => centre[i].1,
                    k => (centre[i].1 + wired[i].iter().map(|&j| centre[j].1).sum::<f32>()) / (k + 1) as f32,
                })
                .collect();
            rule(&mut centre, kind, &wants);
        }
    }
    // A note stands beside what it annotates: right of it if there is room, else left, under or
    // over -- the first spot that is clear.
    let clear = |centre: &[(f32, f32)], i: usize, at: (f32, f32), placed: &[bool]| {
        let (w, h) = bodies[i].size;
        let r = (at.0 - w / 2.0, at.1 - h / 2.0, at.0 + w / 2.0, at.1 + h / 2.0);
        let inside = r.0 >= bounds.left && r.1 >= bounds.top && r.2 <= bounds.right && r.3 <= bounds.bottom;
        let off_blocks = bounds.keep_out.iter().all(|&(l, t, rr, b)| r.2 + GAP <= l || r.0 >= rr + GAP || r.3 + GAP <= t || r.1 >= b + GAP);
        let off_cards = (0..n).filter(|&j| j != i && placed[j]).all(|j| {
            let (bw, bh) = bodies[j].size;
            (at.0 - centre[j].0).abs() >= (w + bw) / 2.0 + GAP || (at.1 - centre[j].1).abs() >= (h + bh) / 2.0 + GAP
        });
        inside && off_blocks && off_cards
    };
    let mut placed: Vec<bool> = (0..n).map(|i| band(bodies[i].kind).is_some()).collect();
    for i in 0..n {
        if placed[i] {
            continue;
        }
        let target = edges.iter().find_map(|&(a, b)| match (a == i, b == i) {
            (true, false) if b < n => Some(b),
            (false, true) if a < n => Some(a),
            _ => None,
        });
        if let Some(t) = target {
            let (w, h) = bodies[i].size;
            let (tw, th) = bodies[t].size;
            let c = centre[t];
            let (dx, dy) = ((w + tw) / 2.0 + GAP * 2.0, (h + th) / 2.0 + GAP * 2.0);
            let spots = [(c.0 + dx, c.1), (c.0 - dx, c.1), (c.0, c.1 + dy), (c.0, c.1 - dy), (c.0 + dx, c.1 + dy), (c.0 + dx, c.1 - dy)];
            if let Some(&spot) = spots.iter().find(|&&spot| clear(&centre, i, spot, &placed)) {
                centre[i] = spot;
            }
        }
        placed[i] = true;
    }
    // Whatever still overlaps, pushed apart along whichever way it overlaps least -- though a card
    // in a column only moves up or down it, and a note gives way to it, unless the column is too
    // crowded for that to settle: a sheet read small, say, where every card and block takes twice
    // the room.
    for round in 0..UNTANGLE {
        let ruled = |i: usize| round < UNTANGLE / 3 && band(bodies[i].kind).is_some();
        let mut moved = false;
        for i in 0..n {
            for j in i + 1..n {
                let (a, b) = (bodies[i].size, bodies[j].size);
                let over_x = (a.0 + b.0) / 2.0 + GAP - (centre[i].0 - centre[j].0).abs();
                let over_y = (a.1 + b.1) / 2.0 + GAP - (centre[i].1 - centre[j].1).abs();
                if over_x <= 0.0 || over_y <= 0.0 {
                    continue;
                }
                moved = true;
                if over_x < over_y && !(ruled(i) && ruled(j)) {
                    let s = if centre[i].0 < centre[j].0 { -1.0 } else { 1.0 };
                    let (mi, mj) = match (ruled(i), ruled(j)) {
                        (true, _) => (0.0, 1.0),
                        (_, true) => (1.0, 0.0),
                        _ => (0.5, 0.5),
                    };
                    centre[i].0 += s * (over_x * mi + 0.5);
                    centre[j].0 -= s * (over_x * mj + 0.5);
                } else {
                    let s = if centre[i].1 < centre[j].1 { -1.0 } else { 1.0 };
                    centre[i].1 += s * over_y / 2.0 + s * 0.5;
                    centre[j].1 -= s * over_y / 2.0 + s * 0.5;
                }
                centre[i] = held(centre[i], a, bounds);
                centre[j] = held(centre[j], b, bounds);
            }
        }
        if !moved {
            break;
        }
    }
    centre
        .iter()
        .zip(bodies)
        .map(|(&(x, y), body)| {
            let corner = (x - body.size.0 / 2.0, y - body.size.1 / 2.0);
            let snapped = (snap(corner.0), snap(corner.1));
            let (x, y) = held(
                (snapped.0 + body.size.0 / 2.0, snapped.1 + body.size.1 / 2.0),
                body.size,
                bounds,
            );
            (snap_inside(x - body.size.0 / 2.0, bounds.left, bounds.right - body.size.0), snap_inside(y - body.size.1 / 2.0, bounds.top, bounds.bottom - body.size.1))
        })
        .collect()
}

/// Where a card `size` across goes that was asked for at `want`, its top left: the nearest place on
/// the grid that stands [`GAP`] clear of every box in `taken` and of all `bounds` keeps out, inside
/// `bounds` -- looked for ring by ring out from `want`, so a card added where the drawing is busy
/// lands beside what is there rather than on it. `want`, held inside the bounds, where nothing
/// near is clear.
pub fn clear(want: (f32, f32), size: (f32, f32), taken: &[(f32, f32, f32, f32)], bounds: &Bounds) -> (f32, f32) {
    const STEP: f32 = 3.0 * SNAP;
    const RINGS: i32 = 48;
    let inside = |(x, y): (f32, f32)| {
        (
            snap_inside(x, bounds.left, (bounds.right - size.0).max(bounds.left)),
            snap_inside(y, bounds.top, (bounds.bottom - size.1).max(bounds.top)),
        )
    };
    let free = |(x, y): (f32, f32)| {
        let (l, t, r, b) = (x - GAP, y - GAP, x + size.0 + GAP, y + size.1 + GAP);
        taken
            .iter()
            .chain(&bounds.keep_out)
            .all(|&(tl, tt, tr, tb)| r <= tl || l >= tr || b <= tt || t >= tb)
    };
    let start = inside(want);
    for ring in 0..=RINGS {
        let mut cells: Vec<(i32, i32)> = (-ring..=ring)
            .flat_map(|dx| (-ring..=ring).map(move |dy| (dx, dy)))
            .filter(|(dx, dy)| dx.abs().max(dy.abs()) == ring)
            .collect();
        cells.sort_by_key(|(dx, dy)| dx * dx + dy * dy);
        for (dx, dy) in cells {
            let at = (start.0 + dx as f32 * STEP, start.1 + dy as f32 * STEP);
            if inside(at) == at && free(at) {
                return at;
            }
        }
    }
    start
}

/// `v` on the 8px grid.
pub fn snap(v: f32) -> f32 {
    (v / SNAP).round() * SNAP
}

/// `v` on the grid and between `low` and `high`, the grid point nearest that is.
fn snap_inside(v: f32, low: f32, high: f32) -> f32 {
    let s = snap(v);
    if s < low {
        (low / SNAP).ceil() * SNAP
    } else if s > high {
        (high / SNAP).floor() * SNAP
    } else {
        s
    }
}

/// A centre kept where a card of `size` may stand: inside the bounds, and pushed off any box kept
/// clear by the shortest way out.
fn held(centre: (f32, f32), size: (f32, f32), bounds: &Bounds) -> (f32, f32) {
    let (hw, hh) = (size.0 / 2.0, size.1 / 2.0);
    let mut x = centre.0.clamp(bounds.left + hw, (bounds.right - hw).max(bounds.left + hw));
    let mut y = centre.1.clamp(bounds.top + hh, (bounds.bottom - hh).max(bounds.top + hh));
    for &(l, t, r, b) in &bounds.keep_out {
        let (l, t, r, b) = (l - hw - GAP / 2.0, t - hh - GAP / 2.0, r + hw + GAP / 2.0, b + hh + GAP / 2.0);
        if x > l && x < r && y > t && y < b {
            let ways = [(x - l, 0), (r - x, 1), (y - t, 2), (b - y, 3)];
            let (_, way) = ways
                .iter()
                .copied()
                .filter(|&(_, way)| match way {
                    0 => l >= bounds.left + hw,
                    1 => r <= bounds.right - hw,
                    2 => t >= bounds.top + hh,
                    _ => b <= bounds.bottom - hh,
                })
                .fold((f32::MAX, 0), |best, next| if next.0 < best.0 { next } else { best });
            match way {
                0 => x = l,
                1 => x = r,
                2 => y = t,
                _ => y = b,
            }
        }
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::{Body, Bounds, GAP, arrange, clear, snap};
    use crate::board::Board;

    fn fixture() -> (Vec<Body>, Vec<(usize, usize)>, Bounds) {
        let board = Board::fixture();
        let bodies: Vec<Body> = board
            .nodes
            .iter()
            .map(|n| Body {
                kind: n.kind,
                at: n.at,
                size: n.kind.footprint(),
            })
            .collect();
        let index = |id| board.nodes.iter().position(|n| n.id == id).unwrap();
        let edges = board.edges.iter().map(|e| (index(e.from), index(e.to))).collect();
        let bounds = Bounds {
            left: 60.0,
            top: 60.0,
            right: 1812.0,
            bottom: 1332.0,
            keep_out: vec![(1420.0, 1196.0, 1836.0, 1356.0), (1476.0, 36.0, 1836.0, 180.0)],
        };
        (bodies, edges, bounds)
    }

    fn overlaps(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
        a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3
    }

    #[test]
    fn everything_lands_inside_the_drawing_and_off_its_blocks() {
        let (bodies, edges, bounds) = fixture();
        let placed = arrange(&bodies, &edges, &bounds);
        assert_eq!(placed.len(), bodies.len());
        for (body, &(x, y)) in bodies.iter().zip(&placed) {
            let r = (x, y, x + body.size.0, y + body.size.1);
            assert!(r.0 >= bounds.left - 0.01 && r.2 <= bounds.right + 0.01, "{r:?} runs off across");
            assert!(r.1 >= bounds.top - 0.01 && r.3 <= bounds.bottom + 0.01, "{r:?} runs off down");
            for &block in &bounds.keep_out {
                assert!(!overlaps(r, block), "{r:?} stands on {block:?}");
            }
        }
    }

    #[test]
    fn no_two_cards_overlap_and_every_corner_is_on_the_grid() {
        let (bodies, edges, bounds) = fixture();
        let placed = arrange(&bodies, &edges, &bounds);
        for i in 0..placed.len() {
            assert_eq!(placed[i].0, snap(placed[i].0));
            assert_eq!(placed[i].1, snap(placed[i].1));
            for j in i + 1..placed.len() {
                let a = (placed[i].0, placed[i].1, placed[i].0 + bodies[i].size.0, placed[i].1 + bodies[i].size.1);
                let b = (placed[j].0, placed[j].1, placed[j].0 + bodies[j].size.0, placed[j].1 + bodies[j].size.1);
                // Snapping can take back up to a grid step of the gap, never all of it.
                let grown = (a.0 - GAP / 2.0 + 8.0, a.1 - GAP / 2.0 + 8.0, a.2 + GAP / 2.0 - 8.0, a.3 + GAP / 2.0 - 8.0);
                assert!(!overlaps(grown, b), "{i} and {j} overlap: {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn the_same_board_arranges_the_same_way() {
        let (bodies, edges, bounds) = fixture();
        assert_eq!(arrange(&bodies, &edges, &bounds), arrange(&bodies, &edges, &bounds));
    }

    #[test]
    fn the_cast_ends_up_left_of_the_milestones() {
        let (bodies, edges, bounds) = fixture();
        let placed = arrange(&bodies, &edges, &bounds);
        let mean = |kind| {
            let xs: Vec<f32> = bodies.iter().zip(&placed).filter(|(b, _)| b.kind == kind).map(|(_, p)| p.0).collect();
            xs.iter().sum::<f32>() / xs.len() as f32
        };
        use crate::board::Kind::*;
        assert!(mean(Person) < mean(Meeting));
        assert!(mean(Meeting) < mean(Spec));
        assert!(mean(Spec) < mean(Milestone));
    }

    #[test]
    fn every_kind_but_the_notes_stands_in_one_column() {
        let (bodies, edges, bounds) = fixture();
        let placed = arrange(&bodies, &edges, &bounds);
        use crate::board::Kind::*;
        for kind in [Person, Meeting, Spec, Milestone] {
            let mut xs: Vec<f32> = bodies.iter().zip(&placed).filter(|(b, _)| b.kind == kind).map(|(_, p)| p.0).collect();
            xs.dedup();
            assert_eq!(xs.len(), 1, "{kind:?} stands at {xs:?}");
        }
    }

    #[test]
    fn a_meeting_with_markup_beside_it_keeps_the_next_column_clear() {
        let (mut bodies, edges, bounds) = fixture();
        // Two meetings carrying callouts out to the right, as wide as the gap to the specs.
        let mut wide = bodies.iter().enumerate().filter(|(_, b)| b.kind == crate::board::Kind::Meeting).map(|(i, _)| i);
        for i in [wide.next().unwrap(), wide.nth(3).unwrap()] {
            bodies[i].size = (bodies[i].size.0 + 520.0, bodies[i].size.1 + 160.0);
        }
        let placed = arrange(&bodies, &edges, &bounds);
        for i in 0..placed.len() {
            for j in i + 1..placed.len() {
                let a = (placed[i].0, placed[i].1, placed[i].0 + bodies[i].size.0, placed[i].1 + bodies[i].size.1);
                let b = (placed[j].0, placed[j].1, placed[j].0 + bodies[j].size.0, placed[j].1 + bodies[j].size.1);
                assert!(!overlaps(a, b), "{i} and {j} overlap: {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn given_room_for_every_scale_and_its_markup_nothing_overlaps() {
        // As the drawing asks: each card the most room it takes at any scale -- in brief at half,
        // in full at one -- four meetings boxed in pencil, and two of them with callouts beside them.
        let (mut bodies, edges, bounds) = fixture();
        for body in &mut bodies {
            let ((fw, fh), (bw, bh)) = (body.kind.footprint(), body.kind.brief());
            body.size = (fw.max(bw * 2.0), fh.max(bh * 2.0));
        }
        let meetings: Vec<usize> = bodies.iter().enumerate().filter(|(_, b)| b.kind == crate::board::Kind::Meeting).map(|(i, _)| i).collect();
        for (n, &i) in meetings.iter().skip(1).step_by(2).take(4).enumerate() {
            // A box 23 out all round at half scale; the one with the delta and the callout 46 out
            // left, 59 up and 250 right.
            let (l, t, r, b) = if n % 2 == 1 { (46.0, 59.0, 250.0, 30.0) } else { (23.0, 23.0, 23.0, 23.0) };
            bodies[i].size = (bodies[i].size.0 + l + r, bodies[i].size.1 + t + b);
        }
        // And the blocks kept clear as they stand read at half, twice the sheet they take at full.
        let bounds = Bounds {
            keep_out: vec![(1836.0 - 840.0, 1356.0 - 352.0, 1844.0, 1364.0), (1836.0 - 728.0, 28.0, 1844.0, 300.0)],
            ..bounds
        };
        let placed = arrange(&bodies, &edges, &bounds);
        for i in 0..placed.len() {
            let a = (placed[i].0, placed[i].1, placed[i].0 + bodies[i].size.0, placed[i].1 + bodies[i].size.1);
            assert!(a.0 >= bounds.left && a.2 <= bounds.right && a.1 >= bounds.top && a.3 <= bounds.bottom, "{i} runs off: {a:?}");
            for &block in &bounds.keep_out {
                assert!(!overlaps(a, block), "{i} stands on {block:?}");
            }
            for j in i + 1..placed.len() {
                let b = (placed[j].0, placed[j].1, placed[j].0 + bodies[j].size.0, placed[j].1 + bodies[j].size.1);
                assert!(!overlaps(a, b), "{i} and {j} overlap: {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn nothing_to_arrange_is_nothing() {
        let (_, _, bounds) = fixture();
        assert!(arrange(&[], &[], &bounds).is_empty());
    }

    /// A card asked for where another stands goes beside it, clear by the gap; one asked for in the
    /// open stays where it was asked for.
    #[test]
    fn a_card_added_on_another_lands_clear_of_it() {
        let bounds = Bounds {
            left: 0.0,
            top: 0.0,
            right: 1000.0,
            bottom: 800.0,
            keep_out: vec![(600.0, 0.0, 1000.0, 200.0)],
        };
        let taken = [(400.0, 400.0, 592.0, 466.0)];
        let size = (192.0, 66.0);
        let at = clear((400.0, 400.0), size, &taken, &bounds);
        let (l, t, r, b) = (at.0, at.1, at.0 + size.0, at.1 + size.1);
        let (tl, tt, tr, tb) = taken[0];
        assert!(r + GAP <= tl || l - GAP >= tr || b + GAP <= tt || t - GAP >= tb, "{at:?} overlaps");
        assert_eq!((snap(at.0), snap(at.1)), at, "on the grid");
        assert_eq!(clear((80.0, 80.0), size, &taken, &bounds), (80.0, 80.0));
        // Never into what is kept out, nor past the edge.
        let at = clear((900.0, 50.0), size, &taken, &bounds);
        assert!(at.0 + size.0 <= 1000.0 && (at.1 >= 200.0 + GAP || at.0 + size.0 + GAP <= 600.0), "{at:?}");
    }
}
