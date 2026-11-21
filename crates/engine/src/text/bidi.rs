//! Bidirectional text (UAX #9): embedding levels for a paragraph whose direction the `direction`
//! style gives, mirrored brackets at odd levels, and each line's runs put in visual order.

use unicode_bidi::{BidiClass, BidiInfo, Level, bidi_class};

use super::Line;

/// Each character's embedding level. The whitespace before a tab, a paragraph separator or the
/// paragraph's end goes back to the paragraph's level.
pub fn levels(chars: &[char], rtl: bool) -> Vec<u8> {
    // Every right-to-left character and bidi control is at U+0590 or above.
    if !rtl && chars.iter().all(|&c| c < '\u{590}') {
        return vec![0; chars.len()];
    }
    let base = if rtl { Level::rtl() } else { Level::ltr() };
    let text: String = chars.iter().collect();
    let info = BidiInfo::new(&text, Some(base));
    let mut levels: Vec<u8> = text
        .char_indices()
        .map(|(b, _)| info.levels[b].number())
        .collect();
    use BidiClass::*;
    let trailing = |c: char| {
        matches!(
            bidi_class(c),
            S | WS | B | LRI | RLI | FSI | PDI | BN | LRE | RLE | LRO | RLO | PDF
        )
    };
    for i in 0..chars.len() {
        if i + 1 == chars.len() || matches!(bidi_class(chars[i]), S | B) {
            for j in (0..=i).rev().take_while(|&j| trailing(chars[j])) {
                levels[j] = base.number();
            }
        }
    }
    levels
}

/// The character drawn for `c` at an odd (right-to-left) level: `(` for `)` and so on.
pub fn mirror(c: char) -> char {
    unicode_bidi_mirroring::get_mirrored(c).unwrap_or(c)
}

/// Puts a line's runs in visual order: from the highest level down to 1, every stretch of runs
/// at that level or above is reversed; then each right-to-left run's glyphs are reversed.
pub fn reorder(line: &mut Line) {
    let max = line.runs.iter().map(|r| r.level).max().unwrap_or(0);
    for level in (1..=max).rev() {
        let mut i = 0;
        while i < line.runs.len() {
            if line.runs[i].level < level {
                i += 1;
                continue;
            }
            let end = (i..line.runs.len())
                .find(|&j| line.runs[j].level < level)
                .unwrap_or(line.runs.len());
            line.runs[i..end].reverse();
            i = end;
        }
    }
    for r in line.runs.iter_mut().filter(|r| r.level % 2 == 1) {
        r.glyphs.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hebrew_in_english_is_one_raised_stretch() {
        let chars: Vec<char> = "ab שלום cd ".chars().collect();
        assert_eq!(levels(&chars, false), [0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0]);
        // In a right-to-left paragraph the Latin rises to 2 and the trailing space falls to 1.
        let chars: Vec<char> = "שלום ab ".chars().collect();
        assert_eq!(levels(&chars, true), [1, 1, 1, 1, 1, 2, 2, 1]);
        assert_eq!(mirror('('), ')');
    }
}
