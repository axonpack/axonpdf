//! English (US) hyphenation with Liang's patterns.

use std::collections::HashMap;
use std::sync::OnceLock;

const PATTERNS: &str = include_str!("en-us.txt");
const MIN_WORD_LENGTH: usize = 5;

struct Entry {
    levels: Vec<u8>,
    anchor_start: bool,
    anchor_end: bool,
    len: usize,
}

type Table = HashMap<Vec<char>, Vec<Entry>>;

fn table() -> &'static (Table, usize) {
    static TABLE: OnceLock<(Table, usize)> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut map: Table = HashMap::new();
        let mut longest = 0;
        for line in PATTERNS.lines().filter(|l| !l.starts_with('%') && !l.trim().is_empty()) {
            let (mut text, mut levels) = (Vec::new(), Vec::new());
            let (mut anchor_start, mut anchor_end, mut after_level) = (false, false, false);
            for (i, ch) in line.chars().enumerate() {
                if ch == '.' {
                    if i == 0 { anchor_start = true } else { anchor_end = true }
                    after_level = false;
                } else if let Some(d) = ch.to_digit(10) {
                    levels.push(d as u8);
                    after_level = true;
                } else {
                    if !after_level {
                        levels.push(0);
                    }
                    text.push(ch);
                    after_level = false;
                }
            }
            if text.is_empty() {
                continue;
            }
            longest = longest.max(text.len());
            let len = text.len();
            map.entry(text).or_default().push(Entry { levels, anchor_start, anchor_end, len });
        }
        (map, longest)
    })
}

/// Whitespace and ASCII punctuation (digits included) end a word; anything else is a letter.
fn is_letter(c: char) -> bool {
    !(c.is_whitespace()
        || ('!'..='@').contains(&c)
        || ('['..='`').contains(&c)
        || ('{'..='~').contains(&c)
        || ('\u{2013}'..='\u{203C}').contains(&c))
}

/// Splits a word into the syllables it may break between. Punctuation stays attached and never
/// breaks; words shorter than five letters stay whole.
pub fn syllables(token: &str) -> Vec<String> {
    let chars: Vec<char> = token.chars().collect();
    let mut parts = vec![String::new()];
    let mut i = 0;
    while i < chars.len() {
        let letter = is_letter(chars[i]);
        let mut end = i + 1;
        while end < chars.len() && is_letter(chars[end]) == letter {
            end += 1;
        }
        if letter {
            let split = split_word(&chars[i..end]);
            parts.last_mut().unwrap().push_str(&split[0]);
            parts.extend(split.into_iter().skip(1));
        } else {
            parts.last_mut().unwrap().extend(&chars[i..end]);
        }
        i = end;
    }
    parts
}

fn split_word(word: &[char]) -> Vec<String> {
    // Lengths count UTF-16 units, as the patterns were made for them.
    let n = word.len();
    if word.iter().map(|c| c.len_utf16()).sum::<usize>() < MIN_WORD_LENGTH {
        return vec![word.iter().collect()];
    }
    let (map, longest) = table();
    let lower: Vec<char> = word.iter().flat_map(|c| c.to_lowercase()).collect();
    if lower.len() != n {
        return vec![word.iter().collect()];
    }
    let mut levels = vec![0u8; n + 1];
    for start in 0..n {
        for end in start + 1..=n.min(start + longest) {
            let Some(entries) = map.get(&lower[start..end]) else { continue };
            for e in entries {
                if (e.anchor_start && start != 0) || (e.anchor_end && start + e.len != n) {
                    continue;
                }
                for (l, &level) in e.levels.iter().enumerate() {
                    levels[start + l] = levels[start + l].max(level);
                }
            }
        }
    }
    levels[0] = 0;
    levels[1] = 0;
    levels[n] = 0;
    levels[n - 1] = 0;
    let mut parts = Vec::new();
    let mut from = 0;
    for (i, &level) in levels.iter().enumerate().skip(1) {
        if level % 2 == 1 {
            parts.push(word[from..i].iter().collect());
            from = i;
        }
    }
    parts.push(word[from..].iter().collect());
    parts
}

#[cfg(test)]
mod tests {
    use super::syllables;

    #[test]
    fn hyphenates() {
        assert_eq!(syllables("hyphenation"), ["hy", "phen", "ation"]);
        assert_eq!(syllables("Typesetting,"), ["Type", "set", "ting,"]);
        assert_eq!(syllables("word"), ["word"]);
        assert_eq!(syllables("  "), ["  "]);
        assert_eq!(syllables("pre-processing"), ["pre-pro", "cess", "ing"]);
        assert_eq!(syllables("extraordinary"), ["ex", "tra", "or", "di", "nary"]);
        assert_eq!(syllables("responsibilities"), ["re", "spon", "si", "bil", "i", "ties"]);
    }
}
