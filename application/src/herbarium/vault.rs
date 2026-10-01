//! The toy vault: a handful of made-up entries, held in memory and nowhere else.
//!
//! What the credentials app keeps sealed on a server, this keeps in a `Vec` for as long as the
//! page is open. The passphrase is printed on the lock, nothing is sealed, and nothing typed goes
//! anywhere. What is real is the shape of it: an entry and its secrets, a generator, a check that
//! finds the shared, the old and the short, and a log of what was done this visit.

/// The passphrase that opens it, said on the lock.
pub(crate) const PASSPHRASE: &str = "moss";

/// What an entry can be.
pub(crate) const KINDS: [&str; 2] = ["login", "note"];

/// How short a password is before the check says so, and how old.
const SHORT: usize = 12;
const OLD: u32 = 365;

/// One entry.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Entry {
    pub(crate) kind: String,
    pub(crate) title: String,
    pub(crate) username: String,
    pub(crate) url: String,
    pub(crate) folder: String,
    pub(crate) password: String,
    pub(crate) otp: String,
    pub(crate) notes: String,
    /// How many days ago the password was set.
    pub(crate) days: u32,
}

/// One of an entry's secrets, as copy asks for it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Secret {
    Password,
    Otp,
}

impl Secret {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Secret::Password => "password",
            Secret::Otp => "otp",
        }
    }
}

/// A line of what a check found, or of what was done: a name, a word for what, and more.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Line {
    pub(crate) name: String,
    pub(crate) what: String,
    pub(crate) more: String,
}

/// The vault: its entries, by a number that is never reused, and what was done to it this visit.
pub(crate) struct Vault {
    entries: Vec<(usize, Entry)>,
    next: usize,
    log: Vec<Line>,
    /// The generator's state.
    seed: u64,
}

impl Vault {
    /// The vault as the page opens it: a few entries, some of them with something to find.
    pub(crate) fn seeded() -> Self {
        let entry = |kind: &str,
                     title: &str,
                     username: &str,
                     url: &str,
                     folder: &str,
                     password: &str,
                     otp: &str,
                     notes: &str,
                     days: u32| Entry {
            kind: kind.to_string(),
            title: title.to_string(),
            username: username.to_string(),
            url: url.to_string(),
            folder: folder.to_string(),
            password: password.to_string(),
            otp: otp.to_string(),
            notes: notes.to_string(),
            days,
        };
        let seeded = [
            entry(
                "login",
                "greenhouse",
                "fern",
                "greenhouse.example",
                "garden",
                "Wx7#qLp2!vN9sTe4",
                "JBSWY3DPEHPK3PXP",
                "the vents open at 24°",
                40,
            ),
            entry(
                "login",
                "seed library",
                "fern@seeds.example",
                "seeds.example",
                "garden",
                "lavender-lavender",
                "",
                "",
                512,
            ),
            entry(
                "login",
                "potting shed",
                "fern",
                "shed.example",
                "garden",
                "lavender-lavender",
                "",
                "the padlock is the old one",
                88,
            ),
            entry(
                "login",
                "orchard co-op",
                "f.moss",
                "orchard.example",
                "co-op",
                "pear2",
                "",
                "",
                3,
            ),
            entry(
                "login",
                "lichen survey",
                "fern.moss",
                "survey.example",
                "field",
                "r9!Kd2#mPq8$Lz4w",
                "KRSXG5CTMVRXEZLU",
                "",
                12,
            ),
            entry(
                "note",
                "pond pump",
                "",
                "",
                "garden",
                "",
                "",
                "model 40-B · filter every spring · the code on the box is 4471",
                0,
            ),
            entry(
                "login",
                "herbarium press",
                "curator",
                "press.example",
                "field",
                "Tq4$wE8!nB2#kR6m",
                "",
                "",
                760,
            ),
        ];
        let mut vault = Self {
            entries: Vec::new(),
            next: 0,
            log: Vec::new(),
            seed: 0x9E37_79B9_7F4A_7C15,
        };
        for entry in seeded {
            vault.insert(entry);
        }
        vault
    }

    fn insert(&mut self, entry: Entry) -> usize {
        let key = self.next;
        self.next += 1;
        self.entries.push((key, entry));
        key
    }

    /// Every entry, in the order they were added.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// The entry `key`, if it is still there.
    pub(crate) fn entry(&self, key: usize) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, entry)| entry)
    }

    /// The entries that have every one of `words` in their title, username, url or folder, any
    /// case, by key, in list order.
    pub(crate) fn find(&self, words: &str) -> Vec<usize> {
        let words: Vec<String> = words.split_whitespace().map(str::to_lowercase).collect();
        self.entries
            .iter()
            .filter(|(_, entry)| {
                let said = [&entry.title, &entry.username, &entry.url, &entry.folder]
                    .map(|field| field.to_lowercase())
                    .join(" ");
                words.iter().all(|word| said.contains(word.as_str()))
            })
            .map(|(key, _)| *key)
            .collect()
    }

    /// Adds `entry`, and says so in the log.
    pub(crate) fn add(&mut self, entry: Entry) -> usize {
        self.logged(&entry.title, "added", "");
        self.insert(entry)
    }

    /// Writes `entry` over `key`. Whether it said anything different.
    pub(crate) fn save(&mut self, key: usize, mut entry: Entry) -> bool {
        let Some((_, was)) = self.entries.iter_mut().find(|(k, _)| *k == key) else {
            return false;
        };
        if *was == entry {
            return false;
        }
        if entry.password != was.password {
            entry.days = 0;
        }
        *was = entry.clone();
        self.logged(&entry.title, "saved", "");
        true
    }

    /// Takes `key` out.
    pub(crate) fn erase(&mut self, key: usize) {
        if let Some(n) = self.entries.iter().position(|(k, _)| *k == key) {
            let (_, entry) = self.entries.remove(n);
            self.logged(&entry.title, "deleted", "");
        }
    }

    /// Says in the log that `secret` of `key` was copied.
    pub(crate) fn copied(&mut self, key: usize, secret: Secret) {
        if let Some(title) = self.entry(key).map(|entry| entry.title.clone()) {
            self.logged(&title, "copied", secret.name());
        }
    }

    /// What was done this visit, the latest first.
    pub(crate) fn log(&self) -> Vec<Line> {
        self.log.iter().rev().cloned().collect()
    }

    fn logged(&mut self, name: &str, what: &str, more: &str) {
        self.log.push(Line {
            name: name.to_string(),
            what: what.to_string(),
            more: more.to_string(),
        });
    }

    /// What a check finds: passwords two entries share, passwords set long ago, short ones, and
    /// logins with none.
    pub(crate) fn health(&self) -> Vec<Line> {
        let line = |entry: &Entry, what: &str, more: String| Line {
            name: entry.title.clone(),
            what: what.to_string(),
            more,
        };
        let logins = || {
            self.entries
                .iter()
                .map(|(_, entry)| entry)
                .filter(|entry| entry.kind == "login")
        };
        let mut found = Vec::new();
        for entry in logins().filter(|entry| !entry.password.is_empty()) {
            let others: Vec<&str> = logins()
                .filter(|other| other.password == entry.password && other.title != entry.title)
                .map(|other| other.title.as_str())
                .collect();
            if !others.is_empty() {
                found.push(line(entry, "shared", format!("with {}", others.join(", "))));
            }
        }
        for entry in logins().filter(|entry| entry.days >= OLD) {
            found.push(line(entry, "old", format!("set {}d ago", entry.days)));
        }
        for entry in logins()
            .filter(|entry| !entry.password.is_empty() && entry.password.chars().count() < SHORT)
        {
            found.push(line(entry, "short", String::new()));
        }
        for entry in logins().filter(|entry| entry.password.is_empty()) {
            found.push(line(entry, "empty", String::new()));
        }
        found
    }

    /// A new password, `length` long, with symbols or not: lower, upper and digits always.
    ///
    /// From a xorshift the vault carries, not from the system's randomness -- a toy's passwords
    /// only have to look like passwords.
    pub(crate) fn generate(&mut self, length: usize, symbols: bool) -> String {
        const LOWER: &str = "abcdefghijkmnopqrstuvwxyz";
        const UPPER: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ";
        const DIGITS: &str = "23456789";
        const SYMBOLS: &str = "!#$%&*+-=?@^_";
        let mut alphabet: Vec<char> = [LOWER, UPPER, DIGITS].concat().chars().collect();
        if symbols {
            alphabet.extend(SYMBOLS.chars());
        }
        (0..length)
            .map(|_| alphabet[(self.next_random() % alphabet.len() as u64) as usize])
            .collect()
    }

    /// Stirs the generator with `by`, so two visits do not make the same passwords.
    pub(crate) fn stir(&mut self, by: u64) {
        self.seed ^= by.wrapping_mul(0x2545_F491_4F6C_DD1D);
        if self.seed == 0 {
            self.seed = 1;
        }
    }

    fn next_random(&mut self) -> u64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_check_finds_what_was_planted() {
        let found = Vault::seeded().health();
        let said = |name: &str, what: &str| found.iter().any(|l| l.name == name && l.what == what);
        assert!(said("seed library", "shared"));
        assert!(said("potting shed", "shared"));
        assert!(said("seed library", "old"));
        assert!(said("herbarium press", "old"));
        assert!(said("orchard co-op", "short"));
        assert!(
            !found.iter().any(|l| l.name == "pond pump"),
            "a note is not checked"
        );
    }

    #[test]
    fn find_takes_every_word_in_any_field() {
        let vault = Vault::seeded();
        assert_eq!(vault.find("").len(), vault.len());
        let found = vault.find("GARDEN fern");
        let titles: Vec<&str> = found
            .iter()
            .map(|&k| vault.entry(k).unwrap().title.as_str())
            .collect();
        assert_eq!(titles, ["greenhouse", "seed library", "potting shed"]);
    }

    #[test]
    fn a_new_password_resets_its_age_and_is_logged() {
        let mut vault = Vault::seeded();
        let key = vault.find("herbarium")[0];
        let mut entry = vault.entry(key).unwrap().clone();
        assert!(!vault.save(key, entry.clone()), "the same is not a change");
        entry.password = vault.generate(24, true);
        assert_eq!(entry.password.chars().count(), 24);
        assert!(vault.save(key, entry));
        assert_eq!(vault.entry(key).unwrap().days, 0);
        assert_eq!(vault.log()[0].what, "saved");
    }

    #[test]
    fn an_erased_entry_is_gone_and_its_key_not_reused() {
        let mut vault = Vault::seeded();
        let key = vault.find("pond")[0];
        vault.erase(key);
        assert!(vault.entry(key).is_none());
        let added = vault.add(Entry {
            title: "new".into(),
            ..Entry::default()
        });
        assert_ne!(added, key);
    }
}
