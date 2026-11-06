//! The studio's planning server, which is not there: the answers it would give, worked out here.
//!
//! Pure functions over the board. The sheet never takes an answer from them directly -- each is
//! handed to a [`Courier`](arboretum::sprout::Courier), so it arrives late, as an answer from a
//! server does, and a failure arrives the same way. A real backend would replace this module and
//! nothing else.
//!
//! The server knows one thing the sheet does not: the calendars of people outside the studio.
//! So a slot the sheet thinks is free can still be refused, with a better one offered back.

use crate::board::{Board, Id, Kind, Slot};

/// What a save came back with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    /// The revision the drawing is at now.
    pub revision: u32,
    /// What the revision block records for it.
    pub note: String,
}

/// Why a save was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejected(pub String);

/// The revision letter for revision `n`: A to Z, then AA, AB and on, as drawings run.
pub fn letter(n: u32) -> String {
    let mut n = n as i64;
    let mut out = Vec::new();
    loop {
        out.push((b'A' + (n % 26) as u8) as char);
        n = n / 26 - 1;
        if n < 0 {
            break;
        }
    }
    out.iter().rev().collect()
}

/// Saves the board as revision `revision + 1`, describing `edits` changes -- or refuses to, where
/// something on it could not be filed.
pub fn sync(board: &Board, revision: u32, edits: &[String]) -> Result<Saved, Rejected> {
    if let Some(node) = board.nodes.iter().find(|n| n.title.trim().is_empty()) {
        let what = match node.kind {
            Kind::Person => "a name",
            Kind::Note => "some words",
            _ => "a title",
        };
        return Err(Rejected(format!("{} needs {what}", node.code)));
    }
    let note = match edits {
        [] => "re-issued".to_string(),
        [one] => one.clone(),
        [first, rest @ ..] => format!("{first} +{}", rest.len()),
    };
    Ok(Saved {
        revision: revision + 1,
        note,
    })
}

/// Who outside the studio is busy, and when: what their own calendars say, which only the server
/// can read.
fn elsewhere(board: &Board, person: Id) -> Vec<Slot> {
    match board.get(person) {
        // Noor is in court on Thursday afternoon.
        Some(node) if node.external => vec![Slot::new(3, 12, 4)],
        _ => Vec::new(),
    }
}

/// Why a meeting could not move where it was asked to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Busy {
    /// Who cannot make it.
    pub who: String,
    /// When the server could move it to instead, if anywhen.
    pub instead: Option<Slot>,
}

impl Busy {
    pub fn words(&self) -> String {
        match self.instead {
            Some(slot) => format!("{} is busy · try {}", self.who, slot.start_label()),
            None => format!("{} is busy all week", self.who),
        }
    }
}

/// Moves `meeting` to `to`, if everyone who goes to it can make it, their calendars elsewhere
/// included. Refused, the server offers the next slot that does suit everyone.
pub fn reschedule(board: &Board, meeting: Id, to: Slot) -> Result<Slot, Busy> {
    let people = board.attendees(meeting);
    let busy: Vec<Slot> = people.iter().flat_map(|&p| elsewhere(board, p)).collect();
    if board.free(to, &people, meeting, &busy) {
        return Ok(to);
    }
    let who = people
        .iter()
        .find(|&&p| {
            let theirs = elsewhere(board, p);
            !board.free(to, &[p], meeting, &theirs)
        })
        .and_then(|&p| board.get(p))
        .map(|node| node.first_name())
        .unwrap_or_else(|| "someone".to_string());
    // Asked from just before the slot that was refused, so the offer is the next that suits.
    let mut trial = board.clone();
    if let Some(node) = trial.get_mut(meeting) {
        node.slot = Some(Slot::new(to.day, to.start.saturating_sub(1), to.len));
    }
    Err(Busy {
        who,
        instead: trial.suggest(meeting, &busy),
    })
}

/// The board as the server holds it at revision C, which is what the sheet opens on.
pub fn load() -> (Board, u32, Vec<(u32, &'static str)>) {
    (
        Board::fixture(),
        2,
        vec![(0, "issued for planning"), (1, "specs 011–016 added"), (2, "week 46 booked")],
    )
}

#[cfg(test)]
mod tests {
    use super::{letter, load, reschedule, sync};
    use crate::board::{Board, Slot};

    fn find(board: &Board, title: &str) -> u32 {
        board.nodes.iter().find(|n| n.title == title).unwrap().id
    }

    #[test]
    fn revisions_are_lettered_as_drawings_are() {
        assert_eq!(letter(0), "A");
        assert_eq!(letter(2), "C");
        assert_eq!(letter(25), "Z");
        assert_eq!(letter(26), "AA");
        assert_eq!(letter(27), "AB");
    }

    #[test]
    fn a_save_is_refused_for_a_nameless_node() {
        let (mut board, rev, _) = load();
        let saved = sync(&board, rev, &["moved P-01".into(), "moved P-02".into()]).unwrap();
        assert_eq!(saved.revision, 3);
        assert_eq!(saved.note, "moved P-01 +1");
        let maya = find(&board, "Maya Okafor");
        board.get_mut(maya).unwrap().title = "  ".into();
        assert_eq!(sync(&board, rev, &[]).unwrap_err().0, "P-01 needs a name");
    }

    #[test]
    fn a_free_slot_is_kept() {
        let board = Board::fixture();
        let sync = find(&board, "type preview sync");
        let to = board.suggest(sync, &[]).unwrap();
        assert_eq!(reschedule(&board, sync, to), Ok(to));
    }

    #[test]
    fn a_calendar_elsewhere_refuses_and_offers_another() {
        let board = Board::fixture();
        let receipt = find(&board, "receipt copy pass");
        // The sheet only knows the board, which says 14:30 is free.
        let asked = board.suggest(receipt, &[]).unwrap();
        assert_eq!(asked, Slot::new(3, 13, 2));
        let busy = reschedule(&board, receipt, asked).unwrap_err();
        assert_eq!(busy.who, "noor");
        assert_eq!(busy.instead, Some(Slot::new(3, 16, 2)));
        assert_eq!(busy.words(), "noor is busy · try THU 16:00");
        // And the offer is taken.
        assert_eq!(reschedule(&board, receipt, Slot::new(3, 16, 2)), Ok(Slot::new(3, 16, 2)));
    }
}
