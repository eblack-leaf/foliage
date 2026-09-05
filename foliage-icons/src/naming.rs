//! File stems into Rust identifiers.

/// Rust keywords that cannot be written as raw identifiers, so a name colliding with one is
/// suffixed instead.
const UNRAWABLE: [&str; 4] = ["crate", "self", "Self", "super"];

/// Every other reserved word, which `r#` makes usable as-is.
const KEYWORDS: [&str; 47] = [
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
    "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized",
    "use", "virtual", "where", "while",
];

/// A file stem as a `snake_case` word: `arrow-up` and `ArrowUp` both become `arrow_up`.
///
/// Anything that is not alphanumeric separates, and a case change inside a word does too, so a
/// set generated from either naming convention reads the same.
pub fn snake_case(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    let mut previous: Option<char> = None;
    for character in stem.chars() {
        if !character.is_alphanumeric() {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
            previous = None;
            continue;
        }
        // A lower-to-upper step is a word boundary the separators did not state.
        if character.is_uppercase()
            && previous.is_some_and(|p| p.is_lowercase() || p.is_numeric())
            && !out.ends_with('_')
        {
            out.push('_');
        }
        out.extend(character.to_lowercase());
        previous = Some(character);
    }
    out.trim_matches('_').to_string()
}

/// A file stem as a struct field: [`snake_case`], made usable where it would not be.
///
/// A name that collides with a keyword is written raw, except for the four that cannot be, which
/// are suffixed. A name opening with a digit is prefixed, and one that reduces to nothing keeps
/// the set compiling rather than emitting an empty field.
pub fn field(stem: &str) -> String {
    let name = snake_case(stem);
    if name.is_empty() {
        return "_unnamed".to_string();
    }
    if UNRAWABLE.contains(&name.as_str()) {
        return format!("{name}_");
    }
    if KEYWORDS.contains(&name.as_str()) {
        return format!("r#{name}");
    }
    if name.starts_with(|c: char| c.is_numeric()) {
        return format!("_{name}");
    }
    name
}

#[cfg(test)]
mod tests {
    use super::{field, snake_case};

    #[test]
    fn separators_and_case_changes_both_break_words() {
        assert_eq!(snake_case("arrow-up"), "arrow_up");
        assert_eq!(snake_case("book_open"), "book_open");
        assert_eq!(snake_case("ArrowUp"), "arrow_up");
        assert_eq!(snake_case("chevrons.left"), "chevrons_left");
        assert_eq!(snake_case("box"), "box");
        assert_eq!(snake_case("x"), "x");
    }

    /// A run of separators is one boundary, and the edges of a name are not boundaries at all.
    #[test]
    fn runs_and_edges_do_not_leave_bare_underscores() {
        assert_eq!(snake_case("--arrow--up--"), "arrow_up");
        assert_eq!(snake_case("_leading"), "leading");
    }

    /// An icon is named by whoever drew it, so a stem that is a keyword or opens with a digit is
    /// ordinary input rather than a mistake -- and every one of them has to compile.
    #[test]
    fn a_name_that_cannot_be_written_plainly_is_made_usable() {
        assert_eq!(field("move"), "r#move");
        assert_eq!(field("type"), "r#type");
        assert_eq!(field("self"), "self_");
        assert_eq!(field("crate"), "crate_");
        assert_eq!(field("2x"), "_2x");
        assert_eq!(field("---"), "_unnamed");
        assert_eq!(field("arrow-up"), "arrow_up");
    }
}
