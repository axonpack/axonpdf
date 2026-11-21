//! Paragraphs into lines for a given width.

use super::breaker::{INFINITY, Item, best_fit, knuth_plass};
use super::finalize::finalize;
use super::{Glyph, Line, LineRun, Options, Paragraph};

/// Spaces stretch by half and shrink by a third of their width.
const STRETCH: f64 = 3.0 / 6.0;
const SHRINK: f64 = 3.0 / 9.0;
/// The width the breaker reserves for an added hyphen, whatever the font.
const HYPHEN_WIDTH: f64 = 5.0;

/// Lays out every paragraph in a box `width` wide and `height` tall (`None` or 0: unbounded).
pub fn layout(paragraphs: &[Paragraph], width: f64, height: Option<f64>, opts: &Options) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut room = height.filter(|h| *h != 0.0 && !h.is_nan()).unwrap_or(f64::INFINITY);
    let mut y = 0.0;
    let mut left = opts.max_lines.unwrap_or(usize::MAX);
    for p in paragraphs {
        if left == 0 {
            break;
        }
        let mut block = paragraph_lines(p, width, y, opts);
        let cut = block.len() > left;
        block.truncate(left);
        left -= block.len();
        let block_height: f64 = block.iter().map(|l| l.height).sum();
        // ponytail: no ellipsis yet when maxLines or the height cuts a paragraph.
        let _ = cut && opts.ellipsis;
        if room >= block_height {
            room -= block_height;
            y += block_height;
            lines.extend(block);
        } else {
            let mut used = 0.0;
            for line in block {
                used += line.height;
                if used >= room {
                    break;
                }
                lines.push(line);
            }
            break;
        }
    }
    lines
}

fn paragraph_lines(p: &Paragraph, width: f64, top: f64, opts: &Options) -> Vec<Line> {
    let Some(first) = p.runs.first() else { return Vec::new() };
    let indent = first.attrs.indent;
    let widths = [width - indent, width];
    let items = items(p, opts);
    let mut tolerance = 4.0;
    let mut breaks = knuth_plass(&items, &widths, tolerance);
    while breaks.is_empty() && tolerance < 50.0 {
        tolerance += 5.0;
        breaks = knuth_plass(&items, &widths, tolerance);
    }
    if breaks.is_empty() || breaks == [0] {
        breaks = best_fit(&items, &widths);
    }

    // Character ranges of each line, and whether it ends in an added hyphen.
    let mut ranges: Vec<(usize, usize, bool)> = Vec::new();
    let mut start = 0;
    for &b in breaks.iter().skip(1) {
        if b == items.len() - 1 {
            continue;
        }
        let (end, hyphen) = match items[b] {
            Item::Penalty { .. } => match items[b - 1] {
                Item::Box { end, .. } | Item::Glue { end, .. } => (end, true),
                Item::Penalty { .. } => (start, true),
            },
            Item::Glue { end, .. } | Item::Box { end, .. } => (end, false),
        };
        ranges.push((start, end, hyphen));
        start = end;
    }
    ranges.push((start, p.chars.len(), false));

    let count = ranges.len();
    let mut y = top;
    ranges
        .into_iter()
        .enumerate()
        .map(|(i, (s, e, hyphen))| {
            let line_indent = if i == 0 { indent } else { 0.0 };
            let runs = slice(p, s, e, hyphen);
            let natural = runs.iter().map(|r| r.height).fold(0.0, f64::max);
            let height = natural.max(first.attrs.line_height.unwrap_or(0.0));
            let line = Line {
                x: line_indent,
                y,
                width: width - line_indent,
                height,
                ascent: 0.0,
                x_advance: 0.0,
                overflow_left: 0.0,
                overflow_right: 0.0,
                runs,
            };
            y += height;
            finalize(line, i == count - 1)
        })
        .collect()
}

fn items(p: &Paragraph, opts: &Options) -> Vec<Item> {
    let advance = |s: usize, e: usize| -> f64 {
        p.runs
            .iter()
            .flat_map(|r| &r.glyphs)
            .filter(|g| g.index >= s && g.index < e)
            .map(|g| g.advance)
            .sum()
    };
    let align = &p.runs[0].attrs.align;
    let penalty = opts.hyphenation_penalty.filter(|x| *x != 0.0).unwrap_or(if align == "justify" { 100.0 } else { 600.0 });
    let mut items = Vec::new();
    for (i, &(s, e)) in p.syllables.iter().enumerate() {
        let width = advance(s, e);
        if p.chars[s..e].iter().all(|c| c.is_whitespace()) {
            items.push(Item::Glue { width, start: s, end: e, stretch: width * STRETCH, shrink: width * SHRINK });
        } else {
            items.push(Item::Box { width, start: s, end: e });
            // Another syllable of the same word follows (anything but a single space).
            if let Some(&(ns, ne)) = p.syllables.get(i + 1)
                && p.chars[ns..ne] != [' ']
            {
                items.push(Item::Penalty { width: HYPHEN_WIDTH, penalty, flagged: 1.0 });
            }
        }
    }
    let end = p.chars.len();
    items.push(Item::Glue { width: 0.0, start: end, end, stretch: INFINITY, shrink: 0.0 });
    items.push(Item::Penalty { width: 0.0, penalty: -INFINITY, flagged: 1.0 });
    items
}

/// The runs of characters `s..e`, with a hyphen added to the last one when asked.
fn slice(p: &Paragraph, s: usize, e: usize, hyphen: bool) -> Vec<LineRun> {
    let mut out: Vec<LineRun> = p
        .runs
        .iter()
        .filter(|r| r.start < e && r.end > s)
        .map(|r| {
            let (a, b) = (r.start.max(s), r.end.min(e));
            let glyphs: Vec<Glyph> = r.glyphs.iter().filter(|g| g.index >= a && g.index < b).copied().collect();
            LineRun {
                attrs: r.attrs.clone(),
                start: a,
                font: r.font,
                text: p.chars[a..b].iter().collect(),
                glyphs,
                ascent: r.ascent,
                descent: r.descent,
                height: r.height(),
                x_advance: 0.0,
            }
        })
        .collect();
    if hyphen && let Some(last) = out.last_mut() {
        let run = p.runs.iter().rfind(|r| r.start < e && r.end > s).unwrap();
        last.glyphs.push(Glyph { index: e, ..run.hyphen });
        last.text.push('-');
    }
    out
}
