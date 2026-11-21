//! From styled fragments to shaped paragraphs: fonts picked per character, words split into
//! syllables, soft hyphens dropped, glyphs shaped once.

use std::rc::Rc;

use super::{Attrs, Glyph, Paragraph, Run, hyphen};
use crate::display::FontId;
use crate::fonts::{Font, FontStore};

/// Text in one style, with the fonts to try in order (Helvetica last).
#[derive(Clone, Debug)]
pub struct Fragment {
    pub text: String,
    pub attrs: Rc<Attrs>,
    pub fonts: Vec<FontId>,
}

const SOFT_HYPHEN: char = '\u{ad}';
/// Line and paragraph separators and the word joiner, dropped when the font cannot draw them.
const IGNORABLE: [char; 3] = ['\u{2028}', '\u{2029}', '\u{2060}'];

pub fn prepare(fonts: &FontStore, fragments: &[Fragment]) -> Vec<Paragraph> {
    // The whole string with each character's fragment.
    let mut chars: Vec<(char, usize)> = Vec::new();
    for (i, f) in fragments.iter().enumerate() {
        let first = f.fonts[0];
        for c in f.text.chars() {
            if !IGNORABLE.contains(&c) || fonts.has_char(first, c) {
                chars.push((c, i));
            }
        }
    }
    // Paragraphs end after each newline.
    // ponytail: empty text gives no lines (height 0); a lone empty string may differ.
    let mut out = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = chars[start..]
            .iter()
            .position(|(c, _)| *c == '\n')
            .map_or(chars.len(), |p| start + p + 1);
        out.push(paragraph(fonts, fragments, &chars[start..end]));
        start = end;
    }
    out
}

fn paragraph(fonts: &FontStore, fragments: &[Fragment], chars: &[(char, usize)]) -> Paragraph {
    let mut last_font: Option<FontId> = None;
    // Runs: same fragment, same font.
    let mut pieces: Vec<(usize, FontId, String)> = Vec::new();
    for &(c, frag) in chars {
        let stack = &fragments[frag].fonts;
        let font = match last_font {
            Some(f) if c == SOFT_HYPHEN => f,
            _ => stack
                .iter()
                .copied()
                .chain(last_font)
                .find(|&f| fonts.has_char(f, c))
                .unwrap_or(*stack.last().unwrap()),
        };
        last_font = Some(font);
        match pieces.last_mut() {
            Some((f, id, text)) if *f == frag && *id == font => text.push(c),
            _ => pieces.push((frag, font, c.to_string())),
        }
    }

    let mut p = Paragraph::default();
    for (frag, font, text) in pieces {
        let start = p.chars.len();
        // Words split at runs of spaces; author's soft hyphens replace the patterns.
        for word in split_spaces(&text) {
            let parts = if word.contains(SOFT_HYPHEN) {
                word.split(SOFT_HYPHEN).map(str::to_string).collect()
            } else {
                hyphen::syllables(word)
            };
            for part in parts {
                let part: String = part.chars().filter(|&c| c != SOFT_HYPHEN).collect();
                let s = p.chars.len();
                p.chars.extend(part.chars());
                p.syllables.push((s, p.chars.len()));
            }
        }
        let run_text: String = p.chars[start..].iter().collect();
        p.runs.push(shape(
            fonts,
            fragments[frag].attrs.clone(),
            font,
            start,
            &run_text,
        ));
    }
    p
}

/// `"a  b c"` → `["a", "  ", "b", " ", "c"]`.
fn split_spaces(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut from = 0;
    let bytes = s.as_bytes();
    for i in 1..=bytes.len() {
        if i == bytes.len() || (bytes[i] == b' ') != (bytes[from] == b' ') {
            out.push(&s[from..i]);
            from = i;
        }
    }
    out.retain(|w| !w.is_empty());
    out
}

fn shape(fonts: &FontStore, attrs: Rc<Attrs>, font: FontId, start: usize, text: &str) -> Run {
    let m = fonts.metrics(font);
    let scale = attrs.size / m.units_per_em as f64;
    // Shaped at one em so advances come back in font units, then scaled here in f64.
    let em = m.units_per_em;
    let char_at: Vec<usize> = text.char_indices().map(|(b, _)| b).collect();
    let chars: Vec<char> = text.chars().collect();
    let glyphs = fonts
        .shape(font, text, em, 0.0, &[])
        .into_iter()
        .map(|g| {
            let i = char_at.partition_point(|&b| b < g.cluster as usize);
            Glyph {
                id: g.id,
                index: start + i,
                advance: g.x_advance as f64 * scale + attrs.letter_spacing,
                x_offset: g.x_offset as f64 * scale,
                y_offset: g.y_offset as f64 * scale,
                whitespace: chars.get(i) == Some(&' '),
            }
        })
        .collect();
    // The hyphen added at a break. The Standard 14 give it a fixed 400 units.
    let mut hyphen = fonts.shape(font, "-", em, 0.0, &[]).first().map_or(
        Glyph {
            id: 0x2d,
            index: 0,
            advance: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
            whitespace: false,
        },
        |g| Glyph {
            id: g.id,
            index: 0,
            advance: g.x_advance as f64 * scale,
            x_offset: 0.0,
            y_offset: 0.0,
            whitespace: false,
        },
    );
    if matches!(fonts.font(font), Font::Standard(_)) {
        hyphen.advance = 400.0 * scale;
    }
    Run {
        start,
        end: start + chars.len(),
        attrs,
        font,
        ascent: m.ascent as f64 * scale,
        descent: m.descent as f64 * scale,
        line_gap: m.line_gap as f64 * scale,
        glyphs,
        hyphen,
    }
}
