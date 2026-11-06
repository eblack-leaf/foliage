//! The board: everything the drawing says, and nothing about how it is drawn.
//!
//! People, meetings, specs, milestones and notes, and the links between them, held as plain data
//! the sheet borrows. What can be asked of the plan -- who is double-booked, where a meeting could
//! go instead, what a link between two things must be, what a search finds -- is answered here, in
//! pure functions, so the answers are tested without a surface.

use std::cmp::Ordering;

/// A node's name on the board. Stable for as long as the board lives, deleted or not.
pub type Id = u32;

/// The five things a plan is drawn from.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Person,
    Meeting,
    Spec,
    Milestone,
    Note,
}

impl Kind {
    pub const ALL: [Kind; 5] = [Kind::Person, Kind::Meeting, Kind::Spec, Kind::Milestone, Kind::Note];

    /// What one is called.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Person => "person",
            Kind::Meeting => "meeting",
            Kind::Spec => "spec",
            Kind::Milestone => "milestone",
            Kind::Note => "note",
        }
    }


    /// Where it stands in [`ALL`](Kind::ALL).
    pub fn index(self) -> usize {
        Kind::ALL.iter().position(|kind| *kind == self).unwrap_or(0)
    }

    /// How large a card of this kind is drawn with everything on it, in sheet pixels at full scale.
    pub fn footprint(self) -> (f32, f32) {
        match self {
            Kind::Person => (184.0, 52.0),
            Kind::Meeting => (192.0, 66.0),
            Kind::Spec => (200.0, 78.0),
            Kind::Milestone => (176.0, 52.0),
            Kind::Note => (184.0, 62.0),
        }
    }

    /// How large it is drawn in brief, when the sheet is zoomed out far enough that only what it is
    /// called is worth drawing.
    pub fn brief(self) -> (f32, f32) {
        match self {
            Kind::Person => (172.0, 30.0),
            Kind::Meeting => (124.0, 22.0),
            Kind::Spec => (100.0, 24.0),
            Kind::Milestone => (96.0, 30.0),
            Kind::Note => (64.0, 22.0),
        }
    }
}

/// The working week the plan covers, as the scale bar labels it.
pub const DAYS: [&str; 5] = ["MON", "TUE", "WED", "THU", "FRI"];
/// Half hours in a working day, 08:00 to 18:00.
pub const HALVES: u8 = 20;
/// Lunch, which nothing is suggested over: 12:00 to 13:00.
const LUNCH: (u8, u8) = (8, 10);

/// When a meeting is: a day of the week, and a run of half hours from 08:00.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub day: u8,
    /// Half hours after 08:00.
    pub start: u8,
    /// How many half hours it runs.
    pub len: u8,
}

impl Slot {
    pub const fn new(day: u8, start: u8, len: u8) -> Self {
        Self { day, start, len }
    }

    /// The half hour it ends at, within its day.
    pub fn end(self) -> u8 {
        self.start + self.len
    }

    /// Whether the two share any half hour.
    pub fn overlaps(self, other: Slot) -> bool {
        self.day == other.day && self.start < other.end() && other.start < self.end()
    }

    /// Where it starts in the week, in half hours from Monday 08:00.
    pub fn week(self) -> u32 {
        self.day as u32 * HALVES as u32 + self.start as u32
    }

    /// Whether it fits inside the working day.
    pub fn fits(self) -> bool {
        self.day < DAYS.len() as u8 && self.len > 0 && self.end() <= HALVES
    }

    /// `TUE 10:00–11:00`.
    pub fn label(self) -> String {
        format!("{} {}–{}", DAYS[self.day as usize % 5], clock(self.start), clock(self.end()))
    }

    /// `TUE 10:00`.
    pub fn start_label(self) -> String {
        format!("{} {}", DAYS[self.day as usize % 5], clock(self.start))
    }

    /// How long it runs, said the short way: `30m`, `1h`, `90m`, `2h`.
    pub fn length(self) -> &'static str {
        match self.len {
            1 => "30m",
            2 => "1h",
            3 => "90m",
            4 => "2h",
            5 => "2.5h",
            _ => "3h+",
        }
    }
}

/// The time `half` half hours after 08:00, as a clock reads it.
pub fn clock(half: u8) -> String {
    let minutes = 8 * 60 + half as u32 * 30;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Where a spec has got to.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Draft,
    Review,
    Approved,
}

impl Status {
    pub const ALL: [Status; 3] = [Status::Draft, Status::Review, Status::Approved];

    /// What the stamp on it says.
    pub fn word(self) -> &'static str {
        match self {
            Status::Draft => "DRAFT",
            Status::Review => "REVIEW",
            Status::Approved => "APPROVED",
        }
    }

    pub fn index(self) -> usize {
        Status::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }
}

/// One thing on the board.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: Id,
    pub kind: Kind,
    /// What it is filed under: `P-04`, `MTG-07`, `SPEC-014`, `MS-2`, `N-1`.
    pub code: String,
    /// A person's name, a meeting's or a spec's title, a note's words.
    pub title: String,
    /// A person's role; nothing for the rest.
    pub role: String,
    /// Where its card's top left corner stands on the sheet, at full scale.
    pub at: (f32, f32),
    /// When a meeting is.
    pub slot: Option<Slot>,
    /// Where a spec has got to.
    pub status: Status,
    /// The day a milestone falls on.
    pub day: u8,
    /// A person from outside the studio, whose calendar the studio cannot see.
    pub external: bool,
}

impl Node {
    fn new(id: Id, kind: Kind, code: &str, title: &str, at: (f32, f32)) -> Self {
        Self {
            id,
            kind,
            code: code.to_string(),
            title: title.to_string(),
            role: String::new(),
            at,
            slot: None,
            status: Status::Draft,
            day: 0,
            external: false,
        }
    }

    /// A person's initials, as their stamp reads.
    pub fn initials(&self) -> String {
        self.title
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }

    /// The first word of a person's name, lowercased, as a callout says it.
    pub fn first_name(&self) -> String {
        self.title.split_whitespace().next().unwrap_or("someone").to_lowercase()
    }
}

/// What a wire between two nodes means.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Link {
    /// A person answers for a spec or a milestone.
    Owns,
    /// A person goes to a meeting.
    Attends,
    /// One spec or milestone cannot be done before another.
    Depends,
    /// A meeting looks over a spec or a milestone.
    Reviews,
    /// A note says something about anything.
    Annotates,
}

impl Link {
    pub const ALL: [Link; 5] = [Link::Owns, Link::Attends, Link::Depends, Link::Reviews, Link::Annotates];

    pub fn name(self) -> &'static str {
        match self {
            Link::Owns => "owns",
            Link::Attends => "attends",
            Link::Depends => "depends",
            Link::Reviews => "reviews",
            Link::Annotates => "annotates",
        }
    }
}

/// A wire on the board.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: Id,
    pub to: Id,
    pub link: Link,
}

/// What a link between two kinds of thing must be, if there can be one, and whether it reads the
/// other way round -- a meeting dragged to a person is the person attending it.
pub fn infer(from: Kind, to: Kind) -> Option<(Link, bool)> {
    use Kind::*;
    let forward = |from: Kind, to: Kind| -> Option<Link> {
        match (from, to) {
            (Person, Meeting) => Some(Link::Attends),
            (Person, Spec) | (Person, Milestone) => Some(Link::Owns),
            (Spec, Spec) | (Spec, Milestone) | (Milestone, Milestone) => Some(Link::Depends),
            (Meeting, Spec) | (Meeting, Milestone) => Some(Link::Reviews),
            (Note, other) if other != Note => Some(Link::Annotates),
            _ => None,
        }
    };
    match forward(from, to) {
        Some(link) => Some((link, false)),
        None => forward(to, from).map(|link| (link, true)),
    }
}

/// Why two nodes could not be linked.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// A wire from a node to itself.
    Itself,
    /// Two kinds of thing nothing links.
    Pair(Kind, Kind),
    /// The link is drawn already.
    Already,
    /// One end is not on the board.
    Missing,
}

impl Refused {
    /// What the readout says.
    pub fn words(self) -> &'static str {
        use Kind::*;
        match self {
            Refused::Itself => "a node can't link to itself",
            Refused::Already => "that link is drawn already",
            Refused::Missing => "nothing there to link to",
            Refused::Pair(Person, Person) => "people don't attend people",
            Refused::Pair(Meeting, Meeting) => "meetings don't nest",
            Refused::Pair(Note, Note) => "a note can't annotate a note",
            Refused::Pair(_, _) => "those two don't link",
        }
    }
}

/// A person due in two places at once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clash {
    pub person: Id,
    /// The meeting that keeps its slot.
    pub keeps: Id,
    /// The meeting a fix moves: the later of the two, or the longer where they start together.
    pub moves: Id,
    /// When they first overlap.
    pub at: Slot,
}

/// The plan.
#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    next: Id,
}

impl Board {
    /// An empty board.
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            next: 1,
        }
    }

    pub fn get(&self, id: Id) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|node| node.id == id)
    }

    /// The code the next node of `kind` will be filed under.
    pub fn next_code(&self, kind: Kind) -> String {
        let highest = |prefix: &str| {
            self.nodes
                .iter()
                .filter(|node| node.kind == kind)
                .filter_map(|node| node.code.strip_prefix(prefix)?.parse::<u32>().ok())
                .max()
                .unwrap_or(0)
        };
        match kind {
            Kind::Person => format!("P-{:02}", highest("P-") + 1),
            Kind::Meeting => format!("MTG-{:02}", highest("MTG-") + 1),
            Kind::Spec => format!("SPEC-{:03}", highest("SPEC-").max(10) + 1),
            Kind::Milestone => format!("MS-{}", highest("MS-") + 1),
            Kind::Note => format!("N-{}", highest("N-") + 1),
        }
    }

    /// Adds a fresh node of `kind` with its card at `at`. Hands back its id.
    pub fn add(&mut self, kind: Kind, at: (f32, f32)) -> Id {
        let id = self.next;
        self.next += 1;
        let code = self.next_code(kind);
        let title = match kind {
            Kind::Person => "new person",
            Kind::Meeting => "new meeting",
            Kind::Spec => "new spec",
            Kind::Milestone => "new milestone",
            Kind::Note => "a new note",
        };
        let mut node = Node::new(id, kind, &code, title, at);
        match kind {
            Kind::Person => node.role = "role".to_string(),
            Kind::Meeting => node.slot = Some(self.first_free(Slot::new(0, 2, 2), &[])),
            Kind::Milestone => node.day = 4,
            _ => {}
        }
        self.nodes.push(node);
        id
    }

    /// Takes a node off the board, and every wire to or from it.
    pub fn remove(&mut self, id: Id) -> Option<(Node, Vec<Edge>)> {
        let n = self.nodes.iter().position(|node| node.id == id)?;
        let node = self.nodes.remove(n);
        let (gone, kept): (Vec<Edge>, Vec<Edge>) = self.edges.iter().partition(|e| e.from == id || e.to == id);
        self.edges = kept;
        Some((node, gone))
    }

    /// Links `a` to `b`, as whatever the two kinds make it, the right way round.
    pub fn connect(&mut self, a: Id, b: Id) -> Result<Edge, Refused> {
        let edge = self.would_link(a, b)?;
        self.edges.push(edge);
        Ok(edge)
    }

    /// What linking `a` to `b` would draw, without drawing it.
    pub fn would_link(&self, a: Id, b: Id) -> Result<Edge, Refused> {
        if a == b {
            return Err(Refused::Itself);
        }
        let (Some(from), Some(to)) = (self.get(a), self.get(b)) else {
            return Err(Refused::Missing);
        };
        let (link, swap) = infer(from.kind, to.kind).ok_or(Refused::Pair(from.kind, to.kind))?;
        let (from, to) = if swap { (b, a) } else { (a, b) };
        let edge = Edge { from, to, link };
        if self.edges.iter().any(|e| e.from == from && e.to == to) {
            return Err(Refused::Already);
        }
        Ok(edge)
    }

    /// Who goes to `meeting`.
    pub fn attendees(&self, meeting: Id) -> Vec<Id> {
        self.edges
            .iter()
            .filter(|e| e.link == Link::Attends && e.to == meeting)
            .map(|e| e.from)
            .collect()
    }

    /// Which meetings `person` goes to.
    pub fn meetings_of(&self, person: Id) -> Vec<Id> {
        self.edges
            .iter()
            .filter(|e| e.link == Link::Attends && e.from == person)
            .map(|e| e.to)
            .collect()
    }

    fn slot_of(&self, meeting: Id) -> Option<Slot> {
        self.get(meeting).and_then(|node| node.slot)
    }

    /// Everyone due in two places at once, earliest first.
    pub fn clashes(&self) -> Vec<Clash> {
        let mut found = Vec::new();
        for person in self.nodes.iter().filter(|node| node.kind == Kind::Person) {
            let mut meetings: Vec<(Id, Slot)> = self
                .meetings_of(person.id)
                .into_iter()
                .filter_map(|m| Some((m, self.slot_of(m)?)))
                .collect();
            meetings.sort_by_key(|(id, slot)| (slot.week(), *id));
            for i in 0..meetings.len() {
                for j in i + 1..meetings.len() {
                    let ((a, sa), (b, sb)) = (meetings[i], meetings[j]);
                    if !sa.overlaps(sb) {
                        continue;
                    }
                    // The later moves; where they start together, the longer.
                    let (keeps, moves) = match sa.start.cmp(&sb.start) {
                        Ordering::Less => (a, b),
                        Ordering::Greater => (b, a),
                        Ordering::Equal if sb.len >= sa.len => (a, b),
                        Ordering::Equal => (b, a),
                    };
                    let start = sa.start.max(sb.start);
                    found.push(Clash {
                        person: person.id,
                        keeps,
                        moves,
                        at: Slot::new(sa.day, start, 1),
                    });
                }
            }
        }
        found.sort_by_key(|clash| (clash.at.week(), clash.person));
        found
    }

    /// What a clash's callout says: `maya: TUE 10:00 ×2`.
    pub fn clash_words(&self, clash: &Clash) -> String {
        let who = self.get(clash.person).map(Node::first_name).unwrap_or_default();
        format!("{who}: {} ×2", clash.at.start_label())
    }

    /// Whether `slot` is free for every one of `people`, leaving `except` out of it, and clear of
    /// `busy` besides -- what someone's calendar elsewhere says.
    pub fn free(&self, slot: Slot, people: &[Id], except: Id, busy: &[Slot]) -> bool {
        if !slot.fits() || busy.iter().any(|b| b.overlaps(slot)) {
            return false;
        }
        people.iter().all(|&person| {
            self.meetings_of(person)
                .into_iter()
                .filter(|&m| m != except)
                .filter_map(|m| self.slot_of(m))
                .all(|other| !other.overlaps(slot))
        })
    }

    /// The next slot `meeting` could move to: as long as it is, after where it starts, its own day
    /// first and the week after, never over lunch, with every attendee free and clear of `busy`.
    /// Nothing, if the week has no room.
    pub fn suggest(&self, meeting: Id, busy: &[Slot]) -> Option<Slot> {
        let now = self.slot_of(meeting)?;
        let people = self.attendees(meeting);
        let lunch = |slot: Slot| slot.start < LUNCH.1 && LUNCH.0 < slot.end();
        let days = (now.day..5).chain(0..now.day);
        for day in days {
            let from = if day == now.day { now.start + 1 } else { 0 };
            for start in from..HALVES {
                let slot = Slot::new(day, start, now.len);
                if !lunch(slot) && self.free(slot, &people, meeting, busy) {
                    return Some(slot);
                }
            }
        }
        None
    }

    fn first_free(&self, from: Slot, people: &[Id]) -> Slot {
        for week in from.week()..5 * HALVES as u32 {
            let slot = Slot::new((week / HALVES as u32) as u8, (week % HALVES as u32) as u8, from.len);
            if self.free(slot, people, Id::MAX, &[]) {
                return slot;
            }
        }
        from
    }

    /// What `query` finds, best first: a code it names exactly, then titles it starts, then words
    /// it starts, then anything it appears in -- a name, a title, a role, a code.
    pub fn search(&self, query: &str) -> Vec<Id> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<(u32, usize, usize, Id)> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(n, node)| {
                let code = node.code.to_lowercase();
                let title = node.title.to_lowercase();
                let role = node.role.to_lowercase();
                let score = if code == query {
                    0
                } else if title.starts_with(&query) {
                    1
                } else if title.split_whitespace().any(|word| word.starts_with(&query)) {
                    2
                } else if code.contains(&query) {
                    3
                } else if title.contains(&query) || role.contains(&query) {
                    4
                } else {
                    return None;
                };
                Some((score, title.len(), n, node.id))
            })
            .collect();
        scored.sort();
        scored.into_iter().map(|(_, _, _, id)| id).collect()
    }

    /// A short account of what `id` is tied to: `attends 2 · owns 1`.
    pub fn ties(&self, id: Id) -> String {
        let mut counts: Vec<(String, usize)> = Vec::new();
        for edge in self.edges.iter().filter(|e| e.from == id || e.to == id) {
            let word = match (edge.link, edge.from == id) {
                (Link::Attends, true) => "attends",
                (Link::Attends, false) => "attended by",
                (Link::Owns, true) => "owns",
                (Link::Owns, false) => "owned by",
                (Link::Depends, true) => "needs",
                (Link::Depends, false) => "gates",
                (Link::Reviews, true) => "reviews",
                (Link::Reviews, false) => "reviewed in",
                (Link::Annotates, true) => "notes",
                (Link::Annotates, false) => "noted",
            };
            match counts.iter_mut().find(|(w, _)| w == word) {
                Some((_, n)) => *n += 1,
                None => counts.push((word.to_string(), 1)),
            }
        }
        if counts.is_empty() {
            return "no links yet".to_string();
        }
        counts
            .iter()
            .map(|(word, n)| format!("{word} {n}"))
            .collect::<Vec<_>>()
            .join(" · ")
    }

    /// The board the sheet opens on: the *orchard launch*, planned by Fernwood Studio. Every name
    /// is made up.
    pub fn fixture() -> Self {
        let mut board = Board::empty();
        let mut id = 0;
        let mut node = |board: &mut Board, kind: Kind, code: &str, title: &str, at: (f32, f32)| -> Id {
            id += 1;
            board.nodes.push(Node::new(id, kind, code, title, at));
            id
        };
        // The cast, down the left of the sheet.
        let people = [
            ("Maya Okafor", "design lead", false),
            ("Jonah Reyes", "engineering lead", false),
            ("Ines Halvorsen", "product", false),
            ("Theo Marchetti", "front end", false),
            ("Priya Nandakumar", "back end", false),
            ("Sol Bergström", "type designer", false),
            ("Wren Adeyemi", "sound designer", false),
            ("Ada Kowalczyk", "quality", false),
            ("Felix Duarte", "copy", false),
            ("Noor Haddad", "counsel · ext.", true),
            ("Kit Lindqvist", "operations", false),
            ("Rafi Mendel", "marketing", false),
        ];
        // Each stands level with the middle of the meetings they go to, so their wires run as
        // nearly across as they can.
        let rows = [2.0, 7.0, 0.0, 4.0, 8.0, 3.0, 5.0, 10.0, 9.0, 6.0, 1.0, 11.0];
        let mut p = Vec::new();
        for (n, (name, role, external)) in people.iter().enumerate() {
            let at = (72.0, 72.0 + 104.0 * rows[n]);
            let id = node(&mut board, Kind::Person, &format!("P-{:02}", n + 1), name, at);
            let person = board.get_mut(id).unwrap();
            person.role = role.to_string();
            person.external = *external;
            p.push(id);
        }
        // The week's meetings, down the middle.
        let meetings = [
            ("kickoff", Slot::new(0, 2, 2)),
            ("checkout review", Slot::new(1, 4, 3)),
            ("type preview sync", Slot::new(1, 5, 2)),
            ("bag drawer walkthrough", Slot::new(1, 10, 2)),
            ("sound preview crit", Slot::new(2, 6, 2)),
            ("licence wording", Slot::new(2, 14, 2)),
            ("search relevance", Slot::new(3, 12, 1)),
            ("receipt copy pass", Slot::new(3, 12, 2)),
            ("qa bash", Slot::new(4, 4, 4)),
            ("go / no-go", Slot::new(4, 16, 2)),
        ];
        let mut m = Vec::new();
        for (n, (title, slot)) in meetings.iter().enumerate() {
            let at = (440.0, 84.0 + 124.0 * n as f32);
            let id = node(&mut board, Kind::Meeting, &format!("MTG-{:02}", n + 1), title, at);
            board.get_mut(id).unwrap().slot = Some(*slot);
            m.push(id);
        }
        // The store's own features, as specs.
        let specs = [
            ("SPEC-011", "bag drawer", Status::Approved),
            ("SPEC-012", "checkout steps", Status::Review),
            ("SPEC-013", "receipt", Status::Draft),
            ("SPEC-014", "search", Status::Review),
            ("SPEC-015", "type previews", Status::Draft),
            ("SPEC-016", "sound previews", Status::Draft),
        ];
        // And each spec level with its owner.
        let slots = [1.0, 3.0, 5.0, 4.0, 0.0, 2.0];
        let mut s = Vec::new();
        for (n, (code, title, status)) in specs.iter().enumerate() {
            let at = (1064.0, 108.0 + 196.0 * slots[n]);
            let id = node(&mut board, Kind::Spec, code, title, at);
            board.get_mut(id).unwrap().status = *status;
            s.push(id);
        }
        let beta = node(&mut board, Kind::Milestone, "MS-1", "public beta", (1400.0, 560.0));
        board.get_mut(beta).unwrap().day = 2;
        let launch = node(&mut board, Kind::Milestone, "MS-2", "orchard launch", (1400.0, 1000.0));
        board.get_mut(launch).unwrap().day = 4;
        let n1 = node(&mut board, Kind::Note, "N-1", "three steps, no more; the licence goes last", (1360.0, 744.0));
        let n2 = node(&mut board, Kind::Note, "N-2", "bring two specimen sheets, printed", (712.0, 472.0));
        let n3 = node(&mut board, Kind::Note, "N-3", "launch copy freezes thursday noon", (1632.0, 872.0));

        let mut link = |from: Id, to: Id, kind: Link| board.edges.push(Edge { from, to, link: kind });
        // Who goes where.
        let attends: [(usize, usize); 18] = [
            (2, 0),
            (10, 0),
            (0, 1),
            (1, 1),
            (9, 1),
            (0, 2),
            (5, 2),
            (3, 3),
            (0, 3),
            (6, 4),
            (9, 5),
            (1, 6),
            (4, 6),
            (1, 7),
            (8, 7),
            (9, 7),
            (7, 8),
            (11, 9),
        ];
        for (person, meeting) in attends {
            link(p[person], m[meeting], Link::Attends);
        }
        // Who answers for what.
        for (person, spec) in [(3, 0), (1, 1), (8, 2), (4, 3), (5, 4), (6, 5)] {
            link(p[person], s[spec], Link::Owns);
        }
        link(p[2], launch, Link::Owns);
        // What waits on what.
        link(s[1], s[0], Link::Depends);
        link(s[2], s[1], Link::Depends);
        for spec in [0, 1, 3] {
            link(s[spec], beta, Link::Depends);
        }
        for spec in [2, 4, 5] {
            link(s[spec], launch, Link::Depends);
        }
        link(beta, launch, Link::Depends);
        // What is looked over where.
        link(m[1], s[1], Link::Reviews);
        link(m[2], s[4], Link::Reviews);
        link(m[6], s[3], Link::Reviews);
        // And what the notes are about.
        link(n1, s[1], Link::Annotates);
        link(n2, m[2], Link::Annotates);
        link(n3, launch, Link::Annotates);
        board.next = id + 1;
        board
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, Kind, Link, Refused, Slot, clock, infer};

    fn find(board: &Board, title: &str) -> u32 {
        board.nodes.iter().find(|n| n.title == title).unwrap().id
    }

    #[test]
    fn the_fixture_opens_with_two_people_double_booked() {
        let board = Board::fixture();
        let clashes = board.clashes();
        assert_eq!(clashes.len(), 2, "{clashes:?}");
        let maya = find(&board, "Maya Okafor");
        assert_eq!(clashes[0].person, maya);
        assert_eq!(clashes[0].moves, find(&board, "type preview sync"));
        assert_eq!(clashes[0].keeps, find(&board, "checkout review"));
        assert_eq!(board.clash_words(&clashes[0]), "maya: TUE 10:30 ×2");
        // Starting together, the longer of the two moves.
        assert_eq!(clashes[1].person, find(&board, "Jonah Reyes"));
        assert_eq!(clashes[1].moves, find(&board, "receipt copy pass"));
    }

    #[test]
    fn a_clash_suggests_the_next_free_slot_past_lunch() {
        let board = Board::fixture();
        let sync = find(&board, "type preview sync");
        // 11:00 is still in the review, 11:30 to 13:00 runs into lunch, and 13:00 is the bag
        // drawer walkthrough Maya is in.
        assert_eq!(board.suggest(sync, &[]), Some(Slot::new(1, 12, 2)));
        assert_eq!(clock(12), "14:00");
        // Something elsewhere on the calendar pushes it on.
        assert_eq!(board.suggest(sync, &[Slot::new(1, 12, 4)]), Some(Slot::new(1, 16, 2)));
    }

    #[test]
    fn moving_the_suggested_meeting_clears_the_clash() {
        let mut board = Board::fixture();
        let sync = find(&board, "type preview sync");
        let to = board.suggest(sync, &[]).unwrap();
        board.get_mut(sync).unwrap().slot = Some(to);
        assert_eq!(board.clashes().len(), 1);
    }

    #[test]
    fn links_are_inferred_from_the_pair_either_way_round() {
        assert_eq!(infer(Kind::Person, Kind::Meeting), Some((Link::Attends, false)));
        assert_eq!(infer(Kind::Meeting, Kind::Person), Some((Link::Attends, true)));
        assert_eq!(infer(Kind::Person, Kind::Spec), Some((Link::Owns, false)));
        assert_eq!(infer(Kind::Spec, Kind::Spec), Some((Link::Depends, false)));
        assert_eq!(infer(Kind::Meeting, Kind::Spec), Some((Link::Reviews, false)));
        assert_eq!(infer(Kind::Note, Kind::Milestone), Some((Link::Annotates, false)));
        assert_eq!(infer(Kind::Spec, Kind::Note), Some((Link::Annotates, true)));
        assert_eq!(infer(Kind::Person, Kind::Person), None);
        assert_eq!(infer(Kind::Note, Kind::Note), None);
    }

    #[test]
    fn connecting_refuses_what_cannot_be_drawn() {
        let mut board = Board::fixture();
        let maya = find(&board, "Maya Okafor");
        let jonah = find(&board, "Jonah Reyes");
        let kickoff = find(&board, "kickoff");
        assert_eq!(board.connect(maya, maya), Err(Refused::Itself));
        assert_eq!(board.connect(maya, jonah), Err(Refused::Pair(Kind::Person, Kind::Person)));
        let edge = board.connect(kickoff, maya).unwrap();
        assert_eq!((edge.from, edge.to, edge.link), (maya, kickoff, Link::Attends));
        assert_eq!(board.connect(maya, kickoff), Err(Refused::Already));
    }

    #[test]
    fn removing_a_node_takes_its_wires() {
        let mut board = Board::fixture();
        let maya = find(&board, "Maya Okafor");
        let before = board.edges.len();
        let (_, gone) = board.remove(maya).unwrap();
        assert_eq!(gone.len(), 3);
        assert_eq!(board.edges.len(), before - 3);
        assert!(board.edges.iter().all(|e| e.from != maya && e.to != maya));
        assert_eq!(board.clashes().len(), 1);
    }

    #[test]
    fn search_ranks_codes_then_titles_then_words() {
        let board = Board::fixture();
        assert_eq!(board.search("spec-014"), vec![find(&board, "search")]);
        let found = board.search("se");
        assert_eq!(found[0], find(&board, "search"), "a title it starts comes first");
        assert!(found.contains(&find(&board, "search relevance")));
        // Roles are searched too.
        assert!(board.search("counsel").contains(&find(&board, "Noor Haddad")));
        assert!(board.search("   ").is_empty());
        assert!(board.search("zebra").is_empty());
    }

    #[test]
    fn new_nodes_are_filed_under_the_next_code() {
        let mut board = Board::fixture();
        assert_eq!(board.next_code(Kind::Spec), "SPEC-017");
        assert_eq!(board.next_code(Kind::Person), "P-13");
        let id = board.add(Kind::Spec, (0.0, 0.0));
        assert_eq!(board.get(id).unwrap().code, "SPEC-017");
        assert_eq!(board.next_code(Kind::Spec), "SPEC-018");
        let meeting = board.add(Kind::Meeting, (0.0, 0.0));
        assert!(board.get(meeting).unwrap().slot.is_some());
    }

    #[test]
    fn slots_say_when_they_are() {
        let slot = Slot::new(1, 4, 3);
        assert_eq!(slot.label(), "TUE 10:00–11:30");
        assert_eq!(slot.start_label(), "TUE 10:00");
        assert!(slot.overlaps(Slot::new(1, 6, 1)));
        assert!(!slot.overlaps(Slot::new(1, 7, 1)));
        assert!(!slot.overlaps(Slot::new(2, 4, 3)));
        assert!(!Slot::new(4, 19, 2).fits());
    }

    #[test]
    fn ties_are_counted_by_how_they_read() {
        let board = Board::fixture();
        assert_eq!(board.ties(find(&board, "Maya Okafor")), "attends 3");
        assert_eq!(board.ties(find(&board, "Jonah Reyes")), "attends 3 · owns 1");
    }
}
