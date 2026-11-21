//! From styled fragments to shaped paragraphs: fonts picked per character, words split into
//! syllables, soft hyphens dropped, glyphs shaped once.

use std::rc::Rc;

use super::{Attrs, Glyph, Paragraph, Run, bidi};
use crate::display::FontId;
use crate::fonts::{Font, FontStore};

/// Text in one style, with the fonts to try in order (Helvetica last).
#[derive(Clone, Debug)]
pub struct Fragment {
    pub text: String,
    pub attrs: Rc<Attrs>,
    pub fonts: Vec<FontId>,
}

use super::hyphen::SOFT_HYPHEN;
/// Line and paragraph separators and the word joiner, dropped when the font cannot draw them.
const IGNORABLE: [char; 3] = ['\u{2028}', '\u{2029}', '\u{2060}'];

/// Splits a word into syllables: the built-in [`hyphen::builtin`] or a document's callback.
pub type Split<'a> = dyn FnMut(&str) -> Vec<String> + 'a;

pub fn prepare(fonts: &FontStore, fragments: &[Fragment], split: &mut Split) -> Vec<Paragraph> {
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
        out.push(paragraph(fonts, fragments, &chars[start..end], split));
        start = end;
    }
    out
}

fn paragraph(
    fonts: &FontStore,
    fragments: &[Fragment],
    chars: &[(char, usize)],
    split: &mut Split,
) -> Paragraph {
    let mut last_font: Option<FontId> = None;
    let text: Vec<char> = chars.iter().map(|x| x.0).collect();
    let levels = bidi::levels(&text, fragments[chars[0].1].attrs.rtl);
    // Runs: same fragment, same font, same bidi level.
    let mut pieces: Vec<(usize, FontId, u8, String)> = Vec::new();
    for (&(c, frag), &level) in chars.iter().zip(&levels) {
        let c = if level % 2 == 1 { bidi::mirror(c) } else { c };
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
            Some((f, id, l, text)) if *f == frag && *id == font && *l == level => text.push(c),
            _ => pieces.push((frag, font, level, c.to_string())),
        }
    }

    let mut p = Paragraph::default();
    for (frag, font, level, text) in pieces {
        let start = p.chars.len();
        // Words split at runs of spaces, then into syllables; soft hyphens are dropped after.
        for word in split_spaces(&text) {
            for part in split(word) {
                let part: String = part.chars().filter(|&c| c != SOFT_HYPHEN).collect();
                let s = p.chars.len();
                p.chars.extend(part.chars());
                p.syllables.push((s, p.chars.len()));
            }
        }
        let run_text: String = p.chars[start..].iter().collect();
        let mut run = shape(fonts, fragments[frag].attrs.clone(), font, start, &run_text);
        run.level = level;
        p.runs.push(run);
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
    let chars: Vec<char> = text.chars().collect();
    // `sub` and `super` raise every glyph by a share of the em. The share is taken of the font's
    // units and then read as thousandths of the font size, so fonts not on a 1000 grid move
    // further; shaped x offsets are scaled by the size twice.
    let raise = match attrs.vertical_align.as_deref() {
        Some("sub") => -0.2,
        Some("super") => 0.4,
        _ => 0.0,
    } * m.units_per_em as f64
        * attrs.size
        / 1000.0;
    let shape_at = |text: &str, from: usize| -> Vec<Glyph> {
        let char_at: Vec<usize> = text.char_indices().map(|(b, _)| b).collect();
        let chars: Vec<char> = text.chars().collect();
        let mut glyphs: Vec<Glyph> = fonts
            .shape(font, text, em, 0.0, &[])
            .into_iter()
            .map(|g| {
                let i = char_at.partition_point(|&b| b < g.cluster as usize);
                Glyph {
                    id: g.id,
                    index: from + i,
                    end: from + chars.len(),
                    advance: g.x_advance as f64 * scale + attrs.letter_spacing,
                    x_offset: g.x_offset as f64 * scale * attrs.size / 1000.0,
                    y_offset: raise,
                    whitespace: chars.get(i) == Some(&' '),
                }
            })
            .collect();
        // Each glyph ends where the next one starts.
        let (mut end, mut next) = (from + chars.len(), from + chars.len());
        for g in glyphs.iter_mut().rev() {
            if g.index < next {
                (end, next) = (next, g.index);
            }
            g.end = end;
        }
        glyphs
    };
    let mut glyphs = shape_at(text, start);
    // An attachment takes its own width in place of the replacement character's.
    // and draws as a space under the image.
    if let Some(a) = &attrs.attachment {
        let space = fonts
            .shape(font, " ", em, 0.0, &[])
            .first()
            .map_or(0, |g| g.id);
        for g in glyphs
            .iter_mut()
            .filter(|g| chars.get(g.index - start) == Some(&'\u{fffc}'))
        {
            g.advance = a.width;
            g.id = space;
        }
    }
    // Ligatures, shaped again on both sides of every cut a line break could make in them.
    let mut splits = Vec::new();
    for g in glyphs.iter().filter(|g| g.end - g.index > 1) {
        for cut in g.index + 1..g.end {
            let piece = |a: usize, b: usize| {
                let s: String = chars[a - start..b - start].iter().collect();
                // Cut pieces take each glyph's own advance: no spacing, no kerning.
                let mut gs = shape_at(&s, a);
                gs.iter_mut()
                    .for_each(|x| x.advance -= attrs.letter_spacing);
                gs
            };
            splits.push((g.index, cut, piece(g.index, cut), piece(cut, g.end)));
        }
    }
    // The hyphen added at a break and the ellipsis of cut text. The Standard 14 give each a
    // fixed 400 units.
    let extra = |s: &str| {
        let g = fonts
            .shape(font, s, em, 0.0, &[])
            .first()
            .copied()
            .unwrap_or_default();
        let standard = matches!(fonts.font(font), Font::Standard(_));
        Glyph {
            id: g.id,
            index: 0,
            end: 0,
            advance: if standard { 400.0 } else { g.x_advance as f64 } * scale,
            x_offset: 0.0,
            y_offset: 0.0,
            whitespace: false,
        }
    };
    let attachment_height = attrs.attachment.map_or(0.0, |a| a.height);
    Run {
        start,
        end: start + chars.len(),
        attrs,
        font,
        level: 0,
        ascent: (m.ascent as f64 * scale).max(attachment_height),
        descent: m.descent as f64 * scale,
        line_gap: m.line_gap as f64 * scale,
        glyphs,
        hyphen: extra("-"),
        ellipsis: extra("\u{2026}"),
        splits,
    }
}
