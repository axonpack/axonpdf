//! Paragraphs into lines for a given width.

use super::bidi;
use super::breaker::{INFINITY, Item, best_fit, knuth_plass};
use super::exclusion::line_rects;
use super::finalize::finalize;
use super::{Glyph, Line, LineRun, Options, Paragraph};

/// Spaces stretch by half and shrink by a third of their width.
const STRETCH: f64 = 3.0 / 6.0;
const SHRINK: f64 = 3.0 / 9.0;
/// The width the breaker reserves for an added hyphen, whatever the font.
const HYPHEN_WIDTH: f64 = 5.0;

/// Lays out every paragraph in a box `width` wide and `height` tall (`None` or 0: unbounded).
pub fn layout(
    paragraphs: &[Paragraph],
    width: f64,
    height: Option<f64>,
    opts: &Options,
) -> Vec<Line> {
    let _time = crate::profile::enter(crate::profile::Phase::Text);
    let mut lines = Vec::new();
    let mut room = height
        .filter(|h| *h != 0.0 && !h.is_nan())
        .unwrap_or(f64::INFINITY);
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
        if room >= block_height {
            if cut && opts.ellipsis {
                ellipsis(&mut block, p);
            }
            room -= block_height;
            y += block_height;
            lines.extend(finish(block));
        } else {
            // Out of room: keep the lines that fit and end the last with an ellipsis.
            let mut used = 0.0;
            block.retain(|l| {
                used += l.height;
                used < room
            });
            ellipsis(&mut block, p);
            lines.extend(finish(block));
            break;
        }
    }
    lines
}

fn finish(block: Vec<Line>) -> impl Iterator<Item = Line> {
    let n = block.len();
    block.into_iter().enumerate().map(move |(i, mut l)| {
        bidi::reorder(&mut l);
        finalize(l, i + 1 == n)
    })
}

/// Drops whitespace from one end of a run; true when nothing is left, so trimming goes on.
fn trim(r: &mut LineRun, front: bool) -> bool {
    let chars: Vec<char> = r.text.chars().collect();
    let (mut a, mut b) = (0, chars.len());
    while a < b && chars[if front { a } else { b - 1 }].is_whitespace() {
        if front { a += 1 } else { b -= 1 }
    }
    let (s, e) = (r.start + a, r.start + b);
    r.glyphs.retain(|g| g.index >= s && g.index < e);
    r.text = chars[a..b].iter().collect();
    r.start = s;
    a == b
}

/// Trims the last line's edge spaces and ends it with an ellipsis.
fn ellipsis(block: &mut [Line], p: &Paragraph) {
    let Some(line) = block.last_mut() else { return };
    for r in line.runs.iter_mut() {
        if !trim(r, true) {
            break;
        }
    }
    for r in line.runs.iter_mut().rev() {
        if !trim(r, false) {
            break;
        }
    }
    let Some(last) = line.runs.last_mut() else {
        return;
    };
    let end = last.start + last.text.chars().count();
    let source = p
        .runs
        .iter()
        .rfind(|r| r.font == last.font && r.start <= last.start)
        .unwrap_or(&p.runs[0]);
    last.glyphs.push(Glyph {
        index: end,
        ..source.ellipsis
    });
    last.text.push('…');
}

fn paragraph_lines(p: &Paragraph, width: f64, top: f64, opts: &Options) -> Vec<Line> {
    let Some(first) = p.runs.first() else {
        return Vec::new();
    };
    let indent = first.attrs.indent;
    // Beside floats each line has a box of its own; past them one box takes every line left.
    let band = p.runs.iter().map(|r| r.height()).fold(0.0, f64::max);
    let rects = line_rects(width, top, band, &opts.exclusions);
    let mut widths: Vec<f64> = rects.iter().map(|r| r.2).collect();
    if widths.len() == 1 {
        widths.insert(0, width - indent);
    } else {
        widths[0] -= indent;
    }
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

    let mut rect = 0;
    let mut y = top;
    ranges
        .into_iter()
        .enumerate()
        .map(|(i, (s, e, hyphen))| {
            let line_indent = if i == 0 { indent } else { 0.0 };
            let runs = slice(p, s, e, hyphen);
            let natural = runs.iter().map(|r| r.height).fold(0.0, f64::max);
            let height = natural.max(first.attrs.line_height.unwrap_or(0.0));
            if y + height > rects[rect].1 + rects[rect].3 && rect + 1 < rects.len() {
                rect += 1;
                y = rects[rect].1;
            }
            let (rx, _, rw, _) = rects[rect];
            let line = Line {
                x: rx + line_indent,
                y,
                width: rw - line_indent,
                height,
                ascent: 0.0,
                x_advance: 0.0,
                overflow_left: 0.0,
                overflow_right: 0.0,
                runs,
            };
            y += height;
            line
        })
        .collect()
}

fn items(p: &Paragraph, opts: &Options) -> Vec<Item> {
    let advance = |s: usize, e: usize| -> f64 {
        p.runs
            .iter()
            .flat_map(|r| &r.glyphs)
            // A ligature counts toward the syllable its last character is in.
            .filter(|g| g.end > s && g.end <= e)
            .map(|g| g.advance)
            .sum()
    };
    let align = &p.runs[0].attrs.align;
    let penalty = opts
        .hyphenation_penalty
        .filter(|x| *x != 0.0)
        .unwrap_or(if align == "justify" { 100.0 } else { 600.0 });
    let mut items = Vec::new();
    for (i, &(s, e)) in p.syllables.iter().enumerate() {
        let width = advance(s, e);
        if p.chars[s..e].iter().all(|c| c.is_whitespace()) {
            items.push(Item::Glue {
                width,
                start: s,
                end: e,
                stretch: width * STRETCH,
                shrink: width * SHRINK,
            });
        } else {
            items.push(Item::Box {
                width,
                start: s,
                end: e,
            });
            // Another syllable of the same word follows (anything but a single space).
            if let Some(&(ns, ne)) = p.syllables.get(i + 1)
                && p.chars[ns..ne] != [' ']
            {
                items.push(Item::Penalty {
                    width: HYPHEN_WIDTH,
                    penalty,
                    flagged: 1.0,
                });
            }
        }
    }
    let end = p.chars.len();
    items.push(Item::Glue {
        width: 0.0,
        start: end,
        end,
        stretch: INFINITY,
        shrink: 0.0,
    });
    items.push(Item::Penalty {
        width: 0.0,
        penalty: -INFINITY,
        flagged: 1.0,
    });
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
            let mut glyphs: Vec<Glyph> = Vec::new();
            for g in r.glyphs.iter().filter(|g| g.index < b && g.end > a) {
                if g.index >= a && g.end <= b {
                    glyphs.push(*g);
                    continue;
                }
                // A ligature cut by the line's edge: its pieces on this side.
                let cut = if g.index < a { a } else { b };
                if let Some((_, _, head, tail)) =
                    r.splits.iter().find(|x| x.0 == g.index && x.1 == cut)
                {
                    glyphs.extend(if g.index < a { tail } else { head });
                }
            }
            LineRun {
                attrs: r.attrs.clone(),
                start: a,
                font: r.font,
                level: r.level,
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
        last.glyphs.push(Glyph {
            index: e,
            end: e + 1,
            ..run.hyphen
        });
        last.text.push('-');
    }
    out
}
