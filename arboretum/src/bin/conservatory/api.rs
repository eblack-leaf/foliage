//! The gallery's backend, which is not there: the answers a server would give, worked out here.
//!
//! Pure functions over the exhibition's own fixtures. The page never calls them for an answer it
//! shows at once -- every one is carried through a [`Courier`](arboretum::sprout::Courier), so it
//! arrives late, as it would from a model queue or a print desk. A real backend would replace this
//! module and nothing else.

use arboretum::genart::{Seed, Style};

use crate::works::Work;

/// A request to grow a picture, accepted.
#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub prompt: String,
    pub style: Style,
    pub steps: u32,
    pub seed: u64,
    /// How many are ahead of it in the queue when it is accepted.
    pub ahead: u32,
}

/// Why the model would not grow it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Nothing to begin from.
    Empty,
    /// A word the model will not grow, and what it says about it.
    Declined(&'static str),
}

impl Refusal {
    /// What the studio says.
    pub fn words(&self) -> &'static str {
        match self {
            Refusal::Empty => "the model needs a word to begin from",
            Refusal::Declined(why) => why,
        }
    }
}

/// Words the model has opinions about.
const DECLINED: [(&str, &str); 4] = [
    ("plastic", "ilex declined: it has never seen plastic grow. try something that roots"),
    ("concrete", "ilex declined: nothing it knows takes root in concrete. yet"),
    ("lawn", "ilex declined: it was trained on herbaria, and nobody presses a lawn"),
    ("weed", "ilex declined: it does not believe in weeds"),
];

/// Accepts or refuses a request to grow `prompt` in `style` over `steps`, the way the model queue
/// would. The seed comes from the words, so the same words grow the same picture.
pub fn generate(prompt: &str, style: Style, steps: u32, queue: u32) -> Result<Job, Refusal> {
    let words = prompt.trim();
    if words.is_empty() {
        return Err(Refusal::Empty);
    }
    let lower = words.to_lowercase();
    if let Some((_, why)) = DECLINED.iter().find(|(word, _)| lower.contains(word)) {
        return Err(Refusal::Declined(why));
    }
    let seed = Seed::of(words).value() ^ (style as u64).wrapping_mul(0x9e37_79b9);
    Ok(Job {
        prompt: words.to_string(),
        style,
        steps: steps.clamp(10, 80),
        seed,
        ahead: queue % 4,
    })
}

/// A print reserved: the certificate that comes back from the print desk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Certificate {
    /// The print's number within its edition, `n/of`.
    pub print: String,
    /// The provenance hash, as printed.
    pub hash: String,
}

/// Reserves the next print of `work`, `asked` being how many this visitor has asked for so far.
pub fn reserve(work: &Work, asked: u32) -> Certificate {
    let (sold, of) = work.edition;
    let n = (sold + 1 + asked).min(of);
    let mut seed = Seed::new(work.seed ^ ((n as u64) << 40));
    let hash: String = (0..4)
        .map(|_| format!("{:04x}", seed.next_u64() & 0xffff))
        .collect::<Vec<_>>()
        .join("·");
    Certificate {
        print: format!("{n}/{of}"),
        hash,
    }
}

/// Whether a print of `work` is still to be had, after `asked` reservations.
pub fn available(work: &Work, asked: u32) -> bool {
    work.edition.0 + asked < work.edition.1
}

/// A few words the model likes, for a visitor who has none of their own.
pub fn suggest(n: u32) -> &'static str {
    const SUGGESTIONS: [&str; 8] = [
        "moss on a sleeping piano",
        "a fern made of tide glass",
        "ember orchids in the rain",
        "neon lichen on a lamp post",
        "honey light through a seed head",
        "a river that remembers being ice",
        "rust roses on a ferry rail",
        "dandelion clock at midnight",
    ];
    SUGGESTIONS[n as usize % SUGGESTIONS.len()]
}

#[cfg(test)]
mod tests {
    use super::{Refusal, available, generate, reserve};
    use crate::works::WORKS;
    use arboretum::genart::Style;

    #[test]
    fn the_same_words_grow_the_same_seed() {
        let a = generate("moss on a piano", Style::Bloom, 35, 0).unwrap();
        let b = generate("  Moss on a piano ", Style::Bloom, 35, 7).unwrap();
        assert_eq!(a.seed, b.seed);
        let c = generate("moss on a piano", Style::Flow, 35, 0).unwrap();
        assert_ne!(a.seed, c.seed, "the style is part of the seed");
    }

    #[test]
    fn nothing_and_plastic_are_refused() {
        assert_eq!(generate("   ", Style::Bloom, 35, 0), Err(Refusal::Empty));
        assert!(matches!(
            generate("a plastic rose", Style::Bloom, 35, 0),
            Err(Refusal::Declined(_))
        ));
    }

    #[test]
    fn steps_are_kept_within_what_the_model_takes() {
        assert_eq!(generate("fern", Style::Bloom, 2, 0).unwrap().steps, 10);
        assert_eq!(generate("fern", Style::Bloom, 500, 0).unwrap().steps, 80);
    }

    #[test]
    fn an_edition_runs_out() {
        let last = &WORKS[11];
        assert_eq!(last.edition, (1, 1));
        assert!(!available(last, 0));
        let open = &WORKS[0];
        assert!(available(open, 0));
        assert_eq!(reserve(open, 0).print, "4/7");
        assert_eq!(reserve(open, 1).print, "5/7");
        assert_ne!(reserve(open, 0).hash, reserve(open, 1).hash);
    }
}
