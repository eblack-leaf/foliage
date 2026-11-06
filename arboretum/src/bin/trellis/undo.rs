//! What has been done to the board: the boards it was before each change, to go back to, and the
//! changes not yet filed with the server.

use crate::board::Board;

/// How many changes back the sheet remembers.
const DEPTH: usize = 40;

#[derive(Default)]
pub struct History {
    past: Vec<(Board, String)>,
    unfiled: Vec<String>,
}

impl History {
    /// Remembers `before`, the board as it was, ahead of a change described as `what`.
    pub fn keep(&mut self, before: &Board, what: impl Into<String>) {
        let what = what.into();
        self.past.push((before.clone(), what.clone()));
        if self.past.len() > DEPTH {
            self.past.remove(0);
        }
        self.unfiled.push(what);
    }

    /// The board as it was before the last change, and what that change was. Undoing is itself a
    /// change to file.
    pub fn undo(&mut self) -> Option<(Board, String)> {
        let (board, what) = self.past.pop()?;
        self.unfiled.push(format!("undid {what}"));
        Some((board, what))
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    /// Whether anything is waiting to be filed.
    pub fn unfiled(&self) -> bool {
        !self.unfiled.is_empty()
    }

    /// Everything waiting to be filed, taken.
    pub fn take(&mut self) -> Vec<String> {
        std::mem::take(&mut self.unfiled)
    }

    /// Puts changes back to be filed again, after a save that did not go through.
    pub fn refile(&mut self, edits: Vec<String>) {
        let mut edits = edits;
        edits.append(&mut self.unfiled);
        self.unfiled = edits;
    }
}

#[cfg(test)]
mod tests {
    use super::History;
    use crate::board::Board;

    #[test]
    fn undo_goes_back_one_change_at_a_time_and_is_filed() {
        let mut history = History::default();
        let mut board = Board::fixture();
        let first = board.clone();
        history.keep(&board, "moved P-01");
        board.nodes[0].at = (500.0, 500.0);
        let second = board.clone();
        history.keep(&board, "moved P-02");
        board.nodes[1].at = (600.0, 600.0);
        assert_eq!(history.take(), vec!["moved P-01", "moved P-02"]);
        assert_eq!(history.undo().unwrap().0, second);
        assert_eq!(history.undo().unwrap().0, first);
        assert!(history.undo().is_none());
        assert_eq!(history.take(), vec!["undid moved P-02", "undid moved P-01"]);
    }
}
