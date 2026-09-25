//! Words cut to the room there is for them.
//!
//! A run fitted before it is written, rather than left to wrap wherever its box happens to end: a
//! glimpse is its lines' worth and no more, a name in a row is one line, a note in a stream is
//! two. The count is in character cells, which is what foliage's one monospaced face measures in,
//! and how many cells there is room for is stated where the run stands -- at the size the app was
//! designed at. Narrower than that, a run wraps again of its own accord, which is the window being
//! made small.

/// `text` as one line of at most `columns` cells, with an ellipsis in the last of them where it
/// was cut. What a name, an address or a place is shown as where the row it stands in is one line.
pub fn cut(text: &str, columns: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= columns {
        return text.to_string();
    }
    let kept: String = text.chars().take(columns.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

/// `lines`, each wrapped at its spaces to `columns` letters -- a word longer than that is
/// broken -- and the first `rows` of what that makes, as one run. Wrapped here rather than left
/// to the run, so that what is written is known to be its lines' worth and no more.
pub fn fit<'a>(lines: impl Iterator<Item = &'a String>, columns: usize, rows: usize) -> String {
    let mut fitted: Vec<String> = Vec::new();
    let mut line = String::new();
    for text in lines {
        for mut word in text.split_whitespace() {
            while word.chars().count() > columns {
                if !line.is_empty() {
                    fitted.push(std::mem::take(&mut line));
                }
                let at = word
                    .char_indices()
                    .nth(columns)
                    .map_or(word.len(), |(at, _)| at);
                fitted.push(word[..at].to_string());
                word = &word[at..];
            }
            let room = columns.saturating_sub(line.chars().count());
            if !line.is_empty() && word.chars().count() + 1 > room {
                fitted.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            fitted.push(std::mem::take(&mut line));
        }
    }
    fitted.truncate(rows);
    fitted.join("\n")
}
