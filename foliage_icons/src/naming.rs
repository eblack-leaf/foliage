/// Converts a kebab/snake-case file stem (`arrow-up`, `book_open`) into a PascalCase
/// Rust identifier (`ArrowUp`, `BookOpen`) suitable for an enum variant.
pub fn pascal_case(stem: &str) -> String {
    stem.split(|c: char| c == '-' || c == '_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::pascal_case;

    #[test]
    fn converts_kebab_case() {
        assert_eq!(pascal_case("arrow-up"), "ArrowUp");
        assert_eq!(pascal_case("chevrons-left"), "ChevronsLeft");
        assert_eq!(pascal_case("box"), "Box");
        assert_eq!(pascal_case("x"), "X");
    }
}
