//! A broken line made final: newline dropped, edge spaces hung outside the box, aligned or
//! justified, and measured.

use super::{Glyph, Line};

pub fn finalize(mut line: Line, last_in_paragraph: bool) -> Line {
    // The newline that ended the paragraph is not drawn.
    if let Some(run) = line.runs.last_mut()
        && run.text.ends_with('\n')
    {
        run.text.pop();
        run.glyphs.pop();
    }

    // Leading and trailing spaces hang outside the line box.
    let overflow_left = edge_spaces(line.runs.iter().map(|r| r.glyphs.iter()));
    let overflow_right = edge_spaces(line.runs.iter().rev().map(|r| r.glyphs.iter().rev()));
    line.x -= overflow_left;
    line.width += overflow_left + overflow_right;
    line.overflow_left = overflow_left;
    line.overflow_right = overflow_right;

    let align = &line.runs.first().map_or("left".to_string(), |r| r.attrs.align.clone());
    let align = if last_in_paragraph && align == "justify" { "left" } else { align.as_str() };
    let width: f64 = line.runs.iter().flat_map(|r| &r.glyphs).map(|g| g.advance).sum();
    let factor = match align {
        "center" => 0.5,
        "right" => 1.0,
        _ => 0.0,
    };
    line.x += (line.width - width).max(0.0) * factor;
    if (align == "justify" || width > line.width) && line.width != width {
        let gap = line.width - width;
        justify(&mut line, gap);
    }

    for run in &mut line.runs {
        run.x_advance = run.glyphs.iter().map(|g| g.advance).sum();
    }
    line.ascent = line.runs.iter().map(|r| r.ascent).fold(0.0, f64::max);
    line.x_advance = line.runs.iter().map(|r| r.x_advance).sum();
    line
}

/// The width of the spaces at one end of a line, crossing runs made only of spaces.
fn edge_spaces<'a, I, G>(runs: I) -> f64
where
    I: Iterator<Item = G>,
    G: Iterator<Item = &'a Glyph> + Clone,
{
    let mut width = 0.0;
    for glyphs in runs {
        let mut all = true;
        for g in glyphs {
            if !g.whitespace {
                all = false;
                break;
            }
            width += g.advance;
        }
        if !all {
            break;
        }
    }
    width
}

#[derive(Clone, Copy)]
struct Factor {
    before: f64,
    after: f64,
    priority: usize,
}

/// Spreads `gap` (negative to shrink) over the line: spaces first, then letters.
fn justify(line: &mut Line, gap: f64) {
    let grow = gap > 0.0;
    let space = if grow { Factor { before: 0.5, after: 0.5, priority: 1 } } else { Factor { before: -0.5, after: -0.5, priority: 1 } };
    let letter = if grow {
        Factor { before: 37.0 / 256.0, after: 37.0 / 256.0, priority: 2 }
    } else {
        Factor { before: -11.0 / 256.0, after: -11.0 / 256.0, priority: 2 }
    };
    let mut factors: Vec<Factor> = Vec::new();
    for run in &line.runs {
        let base = factors.len();
        let n = run.glyphs.len();
        for (i, g) in run.glyphs.iter().enumerate() {
            let mut f = if g.whitespace { space } else { letter };
            if g.whitespace && i == n - 1 {
                f.before = 0.0;
                if i > 0 {
                    factors[base + i - 1].after = 0.0;
                }
            }
            factors.push(f);
        }
    }
    let Some(first) = factors.first_mut() else { return };
    first.before = 0.0;
    factors.last_mut().unwrap().after = 0.0;

    let mut sums = [0.0f64; 4];
    let mut total = 0.0;
    for f in &factors {
        total += f.before + f.after;
        sums[f.priority] += f.before + f.after;
    }
    let mut scale = [0.0f64; 4];
    let mut highest: Option<(usize, f64)> = None;
    let mut remaining = gap;
    for p in 0..4 {
        let sum = sums[p];
        if sum == 0.0 {
            continue;
        }
        highest.get_or_insert((p, sum));
        if remaining.abs() <= sum.abs() {
            scale[p] = remaining / sum;
            remaining = 0.0;
            break;
        }
        scale[p] = 1.0;
        remaining -= sum;
    }
    if remaining > 0.0
        && let Some((p, sum)) = highest
    {
        scale[p] = (sum + (gap - total)) / sum;
    }

    let mut k = 0;
    for run in &mut line.runs {
        for g in &mut run.glyphs {
            let f = factors[k];
            let mut d = f.after * scale[f.priority];
            if let Some(next) = factors.get(k + 1) {
                d += next.before * scale[next.priority];
            }
            g.advance += d;
            k += 1;
        }
    }
}
